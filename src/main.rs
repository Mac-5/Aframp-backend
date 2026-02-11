mod cache;
mod chains;
mod config;
mod database;
mod error;
mod health;
mod logging;
mod middleware;
mod services;

// Imports
use crate::health::{HealthChecker, HealthStatus};
use crate::logging::init_tracing;
use axum::{routing::{get, post, patch}, Json, Router};
use cache::{init_cache_pool, CacheConfig, RedisCache};
use chains::stellar::client::StellarClient;
use chains::stellar::config::StellarConfig;
use database::{init_pool, PoolConfig};
use dotenv::dotenv;
use middleware::logging::{request_logging_middleware, UuidRequestId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tracing::{error, info};
use uuid::Uuid;

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize advanced tracing
    init_tracing();

    dotenv().ok();
    let skip_externals = std::env::var("SKIP_EXTERNALS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        "🚀 Starting Aframp backend service"
    );

    // Log configuration
    info!(
        host = std::env::var("HOST").unwrap_or_else(|_| "unknown".to_string()),
        port = std::env::var("PORT").unwrap_or_else(|_| "unknown".to_string()),
        "Server configuration loaded"
    );

    // Initialize database connection pool
    let db_pool = if skip_externals {
        info!("⏭️  Skipping database initialization (SKIP_EXTERNALS=true)");
        None
    } else {
        info!("📊 Initializing database connection pool...");
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?;

        let db_pool = init_pool(&database_url, Some(PoolConfig::default()))
            .await
            .map_err(|e| {
                error!("Failed to initialize database pool: {}", e);
                e
            })?;

        info!(
            max_connections = db_pool.options().get_max_connections(),
            "✅ Database connection pool initialized"
        );
        Some(db_pool)
    };

    // Initialize cache connection pool
    let redis_cache = if skip_externals {
        info!("⏭️  Skipping Redis initialization (SKIP_EXTERNALS=true)");
        None
    } else {
        info!("🔄 Initializing Redis cache connection pool...");
        let redis_url =
            std::env::var("REDIS_URL").map_err(|_| anyhow::anyhow!("REDIS_URL not set"))?;

        let cache_config = CacheConfig {
            redis_url: redis_url.clone(),
            ..Default::default()
        };

        let cache_pool = init_cache_pool(cache_config).await.map_err(|e| {
            error!("Failed to initialize cache pool: {}", e);
            e
        })?;

        let redis_cache = RedisCache::new(cache_pool);
        info!(redis_url = %redis_url, "✅ Cache connection pool initialized");
        Some(redis_cache)
    };

    // Initialize Stellar client
    let stellar_client = if skip_externals {
        info!("⏭️  Skipping Stellar initialization (SKIP_EXTERNALS=true)");
        None
    } else {
        info!("⭐ Initializing Stellar client...");
        let stellar_config = StellarConfig::from_env().map_err(|e| {
            error!("❌ Failed to load Stellar configuration: {}", e);
            e
        })?;

        info!(
            network = ?stellar_config.network,
            timeout_secs = stellar_config.request_timeout.as_secs(),
            max_retries = stellar_config.max_retries,
            "Stellar configuration loaded"
        );

        let stellar_client = StellarClient::new(stellar_config).map_err(|e| {
            error!("❌ Failed to initialize Stellar client: {}", e);
            e
        })?;

        info!("✅ Stellar client initialized successfully");

        // Health check Stellar
        info!("🏥 Performing Stellar health check...");
        let health_status = stellar_client.health_check().await?;
        if health_status.is_healthy {
            info!(
                response_time_ms = health_status.response_time_ms,
                "✅ Stellar Horizon is healthy"
            );
        } else {
            error!(
                error = health_status
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error"),
                "❌ Stellar Horizon health check failed"
            );
        }

        // Demo functionality
        info!("🧪 Demo: Testing Stellar functionality");
        let test_address = "GCJRI5CIWK5IU67Q6DGA7QW52JDKRO7JEAHQKFNDUJUPEZGURDBX3LDX";

        match stellar_client.account_exists(test_address).await {
            Ok(exists) => {
                if exists {
                    info!(address = test_address, "✅ Test account exists");
                    match stellar_client.get_account(test_address).await {
                        Ok(account) => {
                            info!(
                                account_id = %account.account_id,
                                sequence = account.sequence,
                                balances = account.balances.len(),
                                "✅ Successfully fetched account details"
                            );
                            for balance in &account.balances {
                                info!(
                                    balance = %balance.balance,
                                    asset_type = %balance.asset_type,
                                    "Account balance"
                                );
                            }
                        }
                        Err(e) => info!(error = %e, "⚠️  Account exists but couldn't fetch details"),
                    }
                } else {
                    info!(
                        address = test_address,
                        "ℹ️  Test account does not exist (expected)"
                    );
                }
            }
            Err(e) => info!(error = %e, "ℹ️  Error checking account existence (expected for test)"),
        }

        Some(stellar_client)
    };

    // Initialize health checker
    info!("🏥 Initializing health checker...");
    let health_checker =
        HealthChecker::new(db_pool.clone(), redis_cache.clone(), stellar_client.clone());
    info!("✅ Health checker initialized");

    // Create the application router with logging middleware
    info!("🛣️  Setting up application routes...");
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/health/ready", get(readiness))
        .route("/health/live", get(liveness))
        .route("/api/stellar/account/{address}", get(get_stellar_account))
        .route("/api/trustlines/operations", post(create_trustline_operation))
        .route(
            "/api/trustlines/operations/{id}",
            patch(update_trustline_operation_status),
        )
        .route(
            "/api/trustlines/operations/wallet/{address}",
            get(list_trustline_operations_by_wallet),
        )
        .route("/api/fees/calculate", post(calculate_fee))
        .route("/api/afri/trustlines/check", post(check_afri_trustline))
        .route("/api/afri/trustlines/create", post(create_afri_trustline))
        .route("/api/afri/trustlines/verify", post(verify_afri_trustline))
        .route(
            "/api/afri/trustlines/min-balance",
            post(validate_afri_trustline_balance),
        )
        .route("/api/afri/payments/build", post(build_afri_payment))
        .route("/api/afri/payments/sign", post(sign_afri_payment))
        .route("/api/afri/payments/submit", post(submit_afri_payment))
        .with_state(AppState {
            db_pool,
            redis_cache,
            stellar_client,
            health_checker,
        })
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(UuidRequestId))
                .layer(axum::middleware::from_fn(request_logging_middleware))
                .layer(PropagateRequestIdLayer::x_request_id()),
        );

    info!("✅ Routes configured");

    // Run the server with graceful shutdown
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        error!("❌ Failed to bind to address {}: {}", addr, e);
        e
    })?;

    // Print a prominent banner with server information
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║          🚀 AFRAMP BACKEND SERVER IS RUNNING 🚀             ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!(
        "║  🌐 Server Address:  http://{}                    ║",
        addr
    );
    println!(
        "║  📡 Port:            {}                                  ║",
        port
    );
    println!(
        "║  🏠 Host:            {}                            ║",
        host
    );
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  📡 AVAILABLE ENDPOINTS:                                     ║");
    println!("║                                                              ║");
    println!("║  GET  /                          - Root endpoint            ║");
    println!("║  GET  /health                    - Health check             ║");
    println!("║  GET  /health/ready              - Readiness probe          ║");
    println!("║  GET  /health/live               - Liveness probe           ║");
    println!("║  GET  /api/stellar/account/{{address}} - Stellar account    ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  💡 Try it out:                                              ║");
    println!(
        "║     curl http://{}                                ║",
        addr
    );
    println!("║     curl http://{}/health                        ║", addr);
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    info!(
        address = %addr,
        port = %port,
        "🚀 Server listening on http://{}",
        addr
    );
    info!("✅ Server is ready to accept connections");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("👋 Server shutdown complete");

    Ok(())
}

// Application state
#[derive(Clone)]
struct AppState {
    db_pool: Option<sqlx::PgPool>,
    redis_cache: Option<RedisCache>,
    stellar_client: Option<StellarClient>,
    health_checker: HealthChecker,
}

// Handlers
async fn root() -> &'static str {
    info!("📍 Root endpoint accessed");
    "Welcome to Aframp Backend API"
}

async fn health(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<HealthStatus>, (axum::http::StatusCode, String)> {
    info!("🏥 Health check requested");
    let health_status = state.health_checker.check_health().await;

    // Return 503 if any component is unhealthy
    if matches!(health_status.status, crate::health::HealthState::Unhealthy) {
        error!("❌ Health check failed - service unhealthy");
        Err((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable".to_string(),
        ))
    } else {
        info!("✅ Health check passed");
        Ok(Json(health_status))
    }
}

/// Readiness probe - checks if the service is ready to accept traffic
async fn readiness(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<HealthStatus>, (axum::http::StatusCode, String)> {
    info!("🔍 Readiness probe requested");
    // Readiness checks all dependencies
    let result = health(axum::extract::State(state)).await;
    if result.is_ok() {
        info!("✅ Readiness check passed");
    } else {
        error!("❌ Readiness check failed");
    }
    result
}

/// Liveness probe - checks if the service is alive (basic check)
async fn liveness() -> Result<&'static str, (axum::http::StatusCode, String)> {
    info!("💓 Liveness probe requested");
    // Liveness just checks if the service is running
    info!("✅ Liveness check passed");
    Ok("OK")
}

async fn get_stellar_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Result<String, (axum::http::StatusCode, String)> {
    info!(address = %address, "🔍 Stellar account lookup requested");

    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err((
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration".to_string(),
            ))
        }
    };

    match stellar_client.account_exists(&address).await {
        Ok(exists) => {
            if exists {
                info!(address = %address, "✅ Account exists, fetching details");
                match stellar_client.get_account(&address).await {
                    Ok(account) => {
                        info!(
                            address = %address,
                            balances = account.balances.len(),
                            "✅ Account details fetched successfully"
                        );
                        Ok(format!(
                            "Account: {}, Balances: {}",
                            account.account_id,
                            account.balances.len()
                        ))
                    }
                    Err(e) => {
                        error!(address = %address, error = %e, "❌ Failed to fetch account details");
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to fetch account: {}", e),
                        ))
                    }
                }
            } else {
                info!(address = %address, "ℹ️  Account not found");
                Err((
                    axum::http::StatusCode::NOT_FOUND,
                    "Account not found".to_string(),
                ))
            }
        }
        Err(e) => {
            error!(address = %address, error = %e, "❌ Error checking account existence");
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error checking account: {}", e),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct TrustlineOperationRequest {
    wallet_address: String,
    asset_code: String,
    issuer: Option<String>,
    operation_type: TrustlineOperationType,
    status: TrustlineOperationStatus,
    transaction_hash: Option<String>,
    error_message: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TrustlineOperationStatusUpdate {
    status: TrustlineOperationStatus,
    transaction_hash: Option<String>,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TrustlineOperationQuery {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TrustlineOperationType {
    Create,
    Update,
    Remove,
}

impl TrustlineOperationType {
    fn as_str(&self) -> &'static str {
        match self {
            TrustlineOperationType::Create => "create",
            TrustlineOperationType::Update => "update",
            TrustlineOperationType::Remove => "remove",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TrustlineOperationStatus {
    Pending,
    Completed,
    Failed,
}

impl TrustlineOperationStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TrustlineOperationStatus::Pending => "pending",
            TrustlineOperationStatus::Completed => "completed",
            TrustlineOperationStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Deserialize)]
struct FeeCalculationRequest {
    fee_type: FeeType,
    amount: String,
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FeeType {
    Onramp,
    Offramp,
    BillPayment,
    Exchange,
    Transfer,
}

impl FeeType {
    fn as_str(&self) -> &'static str {
        match self {
            FeeType::Onramp => "onramp",
            FeeType::Offramp => "offramp",
            FeeType::BillPayment => "bill_payment",
            FeeType::Exchange => "exchange",
            FeeType::Transfer => "transfer",
        }
    }
}

#[derive(Debug, Serialize)]
struct FeeCalculationResponse {
    fee: String,
    rate_bps: i32,
    flat_fee: String,
    min_fee: Option<String>,
    max_fee: Option<String>,
    currency: Option<String>,
    structure_id: String,
}

#[derive(Debug, Deserialize)]
struct TrustlineAccountRequest {
    account_id: String,
}

#[derive(Debug, Serialize)]
struct TrustlineVerificationResponse {
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct PaymentBuildRequest {
    source: String,
    destination: String,
    amount: String,
    asset_code: String,
    asset_issuer: String,
    memo: Option<crate::services::afri_payment_builder::PaymentMemo>,
    fee_stroops: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PaymentSignRequest {
    draft: crate::services::afri_payment_builder::PaymentTransactionDraft,
    secret_seed: String,
}

#[derive(Debug, Deserialize)]
struct PaymentSubmitRequest {
    draft: crate::services::afri_payment_builder::PaymentTransactionDraft,
    secret_seed: String,
}

#[derive(Debug, Serialize)]
struct PaymentSubmitResponse {
    signed: crate::services::afri_payment_builder::SignedPaymentTransaction,
    horizon_response: serde_json::Value,
}

async fn create_trustline_operation(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineOperationRequest>,
) -> Result<
    Json<crate::database::trustline_operation_repository::TrustlineOperation>,
    (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>),
> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let pool = match state.db_pool.as_ref() {
        Some(pool) => pool,
        None => return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Database disabled by configuration",
            request_id,
        )),
    };

    if payload.wallet_address.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "wallet_address is required",
            request_id,
        ));
    }
    if payload.asset_code.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "asset_code is required",
            request_id,
        ));
    }

    let repo = crate::database::trustline_operation_repository::TrustlineOperationRepository::new(
        pool.clone(),
    );
    let service = crate::services::trustline_operation::TrustlineOperationService::new(repo);

    let input = crate::services::trustline_operation::TrustlineOperationInput {
        wallet_address: payload.wallet_address,
        asset_code: payload.asset_code,
        issuer: payload.issuer,
        operation_type: payload.operation_type.as_str().to_string(),
        status: payload.status.as_str().to_string(),
        transaction_hash: payload.transaction_hash,
        error_message: payload.error_message,
        metadata: payload.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    let result = match payload.operation_type {
        TrustlineOperationType::Create => service.record_create(input).await,
        TrustlineOperationType::Update => service.record_update(input).await,
        TrustlineOperationType::Remove => service.record_remove(input).await,
    };

    result
        .map(Json)
        .map_err(|e| {
            crate::middleware::error::json_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                request_id,
            )
        })
}

async fn update_trustline_operation_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineOperationStatusUpdate>,
) -> Result<
    Json<crate::database::trustline_operation_repository::TrustlineOperation>,
    (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>),
> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let pool = match state.db_pool.as_ref() {
        Some(pool) => pool,
        None => return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Database disabled by configuration",
            request_id,
        )),
    };

    let uuid = Uuid::parse_str(&id).map_err(|e| {
        crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid UUID: {}", e),
            request_id.clone(),
        )
    })?;

    let repo = crate::database::trustline_operation_repository::TrustlineOperationRepository::new(
        pool.clone(),
    );
    let service = crate::services::trustline_operation::TrustlineOperationService::new(repo);

    service
        .update_status(
            uuid,
            payload.status.as_str(),
            payload.transaction_hash.as_deref(),
            payload.error_message.as_deref(),
        )
        .await
        .map(Json)
        .map_err(|e| {
            crate::middleware::error::json_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                request_id.clone(),
            )
        })
}

async fn list_trustline_operations_by_wallet(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(address): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<TrustlineOperationQuery>,
) -> Result<
    Json<Vec<crate::database::trustline_operation_repository::TrustlineOperation>>,
    (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>),
> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let pool = match state.db_pool.as_ref() {
        Some(pool) => pool,
        None => return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Database disabled by configuration",
            request_id,
        )),
    };

    if address.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "wallet address is required",
            request_id,
        ));
    }

    let repo = crate::database::trustline_operation_repository::TrustlineOperationRepository::new(
        pool.clone(),
    );

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    repo.find_by_wallet(&address, limit)
        .await
        .map(Json)
        .map_err(|e| {
            crate::middleware::error::json_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                request_id,
            )
        })
}

async fn calculate_fee(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<FeeCalculationRequest>,
) -> Result<Json<FeeCalculationResponse>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let pool = match state.db_pool.as_ref() {
        Some(pool) => pool,
        None => return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "Database disabled by configuration",
            request_id,
        )),
    };

    let repo = crate::database::fee_structure_repository::FeeStructureRepository::new(
        pool.clone(),
    );
    let service = crate::services::fee_structure::FeeStructureService::new(repo);

    let amount = crate::services::fee_structure::parse_amount(&payload.amount);
    if amount <= bigdecimal::BigDecimal::from(0) {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "amount must be greater than 0",
            request_id,
        ));
    }

    let result = service
        .calculate_fee(crate::services::fee_structure::FeeCalculationInput {
            fee_type: payload.fee_type.as_str().to_string(),
            amount,
            currency: payload.currency,
            at_time: None,
        })
        .await
        .map_err(|e| {
            crate::middleware::error::json_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                request_id.clone(),
            )
        })?;

    match result {
        Some(calc) => Ok(Json(FeeCalculationResponse {
            fee: calc.fee.to_string(),
            rate_bps: calc.rate_bps,
            flat_fee: calc.flat_fee.to_string(),
            min_fee: calc.min_fee.map(|v| v.to_string()),
            max_fee: calc.max_fee.map(|v| v.to_string()),
            currency: calc.currency,
            structure_id: calc.structure_id.to_string(),
        })),
        None => Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::NOT_FOUND,
            "No active fee structure found",
            request_id.clone(),
        )),
    }
}

fn app_error_response(
    err: crate::error::AppError,
    request_id: Option<String>,
) -> (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>) {
    let err = match request_id {
        Some(req_id) => err.with_request_id(req_id),
        None => err,
    };
    let status = axum::http::StatusCode::from_u16(err.status_code())
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        Json(crate::middleware::error::ErrorResponse::from_app_error(&err)),
    )
}

async fn check_afri_trustline(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineAccountRequest>,
) -> Result<Json<crate::services::afri_trustline::TrustlineStatus>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    if payload.account_id.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "account_id is required",
            request_id,
        ));
    }

    let manager =
        crate::services::afri_trustline::TrustlineManager::new(stellar_client.clone());
    manager
        .check_trustline(&payload.account_id)
        .await
        .map(Json)
        .map_err(|e| app_error_response(e, request_id))
}

async fn create_afri_trustline(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineAccountRequest>,
) -> Result<Json<crate::services::afri_trustline::TrustlineTransaction>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    if payload.account_id.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "account_id is required",
            request_id,
        ));
    }

    let manager =
        crate::services::afri_trustline::TrustlineManager::new(stellar_client.clone());
    manager
        .create_trustline_tx(&payload.account_id)
        .await
        .map(Json)
        .map_err(|e| app_error_response(e, request_id))
}

async fn verify_afri_trustline(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineAccountRequest>,
) -> Result<Json<TrustlineVerificationResponse>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    if payload.account_id.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "account_id is required",
            request_id,
        ));
    }

    let manager =
        crate::services::afri_trustline::TrustlineManager::new(stellar_client.clone());
    manager
        .verify_trustline(&payload.account_id)
        .await
        .map(|verified| Json(TrustlineVerificationResponse { verified }))
        .map_err(|e| app_error_response(e, request_id))
}

async fn validate_afri_trustline_balance(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrustlineAccountRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    if payload.account_id.trim().is_empty() {
        return Err(crate::middleware::error::json_error_response(
            axum::http::StatusCode::BAD_REQUEST,
            "account_id is required",
            request_id,
        ));
    }

    let manager =
        crate::services::afri_trustline::TrustlineManager::new(stellar_client.clone());
    manager
        .validate_min_balance(&payload.account_id)
        .await
        .map(|_| Json(serde_json::json!({ "ok": true })))
        .map_err(|e| app_error_response(e, request_id))
}

async fn build_afri_payment(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<PaymentBuildRequest>,
) -> Result<Json<crate::services::afri_payment_builder::PaymentTransactionDraft>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    let builder =
        crate::services::afri_payment_builder::AfriPaymentBuilder::new(stellar_client.clone());
    let operation = crate::services::afri_payment_builder::PaymentOperation {
        source: payload.source,
        destination: payload.destination,
        amount: payload.amount,
        asset_code: payload.asset_code,
        asset_issuer: payload.asset_issuer,
    };

    builder
        .build_payment(
            operation,
            payload
                .memo
                .unwrap_or(crate::services::afri_payment_builder::PaymentMemo::None),
            payload.fee_stroops,
        )
        .await
        .map(Json)
        .map_err(|e| app_error_response(e, request_id))
}

async fn sign_afri_payment(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<PaymentSignRequest>,
) -> Result<Json<crate::services::afri_payment_builder::SignedPaymentTransaction>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    let builder =
        crate::services::afri_payment_builder::AfriPaymentBuilder::new(stellar_client.clone());
    builder
        .sign_transaction(payload.draft, &payload.secret_seed)
        .map(Json)
        .map_err(|e| app_error_response(e, request_id))
}

async fn submit_afri_payment(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<PaymentSubmitRequest>,
) -> Result<Json<PaymentSubmitResponse>, (axum::http::StatusCode, Json<crate::middleware::error::ErrorResponse>)> {
    let request_id = crate::middleware::error::get_request_id_from_headers(&headers);
    let stellar_client = match state.stellar_client.as_ref() {
        Some(client) => client,
        None => {
            return Err(crate::middleware::error::json_error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Stellar client disabled by configuration",
                request_id,
            ))
        }
    };

    let builder =
        crate::services::afri_payment_builder::AfriPaymentBuilder::new(stellar_client.clone());
    let signed = builder
        .sign_transaction(payload.draft, &payload.secret_seed)
        .map_err(|e| app_error_response(e, request_id.clone()))?;

    let horizon_response = stellar_client
        .submit_transaction_xdr(&signed.envelope_xdr)
        .await
        .map_err(|e| {
            app_error_response(
                crate::error::AppError::new(crate::error::AppErrorKind::External(
                    crate::error::ExternalError::Blockchain {
                        message: e.to_string(),
                        is_retryable: true,
                    },
                )),
                request_id,
            )
        })?;

    Ok(Json(PaymentSubmitResponse {
        signed,
        horizon_response,
    }))
}
