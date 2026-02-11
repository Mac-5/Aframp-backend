use crate::database::error::{DatabaseError, DatabaseErrorKind};
use crate::database::repository::{Repository, TransactionalRepository};
use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Conversion audit entity
#[derive(Debug, Clone, FromRow)]
pub struct ConversionAudit {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub wallet_address: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub from_currency: String,
    pub to_currency: String,
    pub from_amount: sqlx::types::BigDecimal,
    pub to_amount: sqlx::types::BigDecimal,
    pub rate: sqlx::types::BigDecimal,
    pub fee_amount: sqlx::types::BigDecimal,
    pub fee_currency: Option<String>,
    pub provider: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Repository for conversion audit trail
pub struct ConversionAuditRepository {
    pool: PgPool,
}

impl ConversionAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a conversion audit record
    pub async fn create(
        &self,
        user_id: Option<Uuid>,
        wallet_address: Option<&str>,
        transaction_id: Option<Uuid>,
        from_currency: &str,
        to_currency: &str,
        from_amount: sqlx::types::BigDecimal,
        to_amount: sqlx::types::BigDecimal,
        rate: sqlx::types::BigDecimal,
        fee_amount: sqlx::types::BigDecimal,
        fee_currency: Option<&str>,
        provider: Option<&str>,
        status: &str,
        error_message: Option<&str>,
        metadata: serde_json::Value,
    ) -> Result<ConversionAudit, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "INSERT INTO conversion_audits 
             (user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) 
             RETURNING id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at",
        )
        .bind(user_id)
        .bind(wallet_address)
        .bind(transaction_id)
        .bind(from_currency)
        .bind(to_currency)
        .bind(from_amount)
        .bind(to_amount)
        .bind(rate)
        .bind(fee_amount)
        .bind(fee_currency)
        .bind(provider)
        .bind(status)
        .bind(error_message)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    /// Update status and optional error message
    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<ConversionAudit, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "UPDATE conversion_audits 
             SET status = $2, error_message = $3, updated_at = NOW() 
             WHERE id = $1 
             RETURNING id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at",
        )
        .bind(id)
        .bind(status)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    /// Find audits by user
    pub async fn find_by_user_id(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ConversionAudit>, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "SELECT id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at 
             FROM conversion_audits 
             WHERE user_id = $1 
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    /// Find audits by transaction id
    pub async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> Result<Vec<ConversionAudit>, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "SELECT id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at 
             FROM conversion_audits 
             WHERE transaction_id = $1 
             ORDER BY created_at DESC",
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }
}

#[async_trait]
impl Repository for ConversionAuditRepository {
    type Entity = ConversionAudit;

    async fn find_by_id(&self, id: &str) -> Result<Option<Self::Entity>, DatabaseError> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            DatabaseError::new(DatabaseErrorKind::Unknown {
                message: format!("Invalid UUID: {}", e),
            })
        })?;
        sqlx::query_as::<_, ConversionAudit>(
            "SELECT id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at 
             FROM conversion_audits WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "SELECT id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at 
             FROM conversion_audits ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn insert(&self, entity: &Self::Entity) -> Result<Self::Entity, DatabaseError> {
        sqlx::query_as::<_, ConversionAudit>(
            "INSERT INTO conversion_audits 
             (id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) 
             RETURNING id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at",
        )
        .bind(entity.id)
        .bind(entity.user_id)
        .bind(&entity.wallet_address)
        .bind(entity.transaction_id)
        .bind(&entity.from_currency)
        .bind(&entity.to_currency)
        .bind(entity.from_amount.clone())
        .bind(entity.to_amount.clone())
        .bind(entity.rate.clone())
        .bind(entity.fee_amount.clone())
        .bind(&entity.fee_currency)
        .bind(&entity.provider)
        .bind(&entity.status)
        .bind(&entity.error_message)
        .bind(&entity.metadata)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn update(&self, id: &str, entity: &Self::Entity) -> Result<Self::Entity, DatabaseError> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            DatabaseError::new(DatabaseErrorKind::Unknown {
                message: format!("Invalid UUID: {}", e),
            })
        })?;
        sqlx::query_as::<_, ConversionAudit>(
            "UPDATE conversion_audits 
             SET user_id = $1, wallet_address = $2, transaction_id = $3, from_currency = $4, to_currency = $5, from_amount = $6, to_amount = $7, rate = $8, fee_amount = $9, fee_currency = $10, provider = $11, status = $12, error_message = $13, metadata = $14, updated_at = NOW()
             WHERE id = $15
             RETURNING id, user_id, wallet_address, transaction_id, from_currency, to_currency, from_amount, to_amount, rate, fee_amount, fee_currency, provider, status, error_message, metadata, created_at, updated_at",
        )
        .bind(entity.user_id)
        .bind(&entity.wallet_address)
        .bind(entity.transaction_id)
        .bind(&entity.from_currency)
        .bind(&entity.to_currency)
        .bind(entity.from_amount.clone())
        .bind(entity.to_amount.clone())
        .bind(entity.rate.clone())
        .bind(entity.fee_amount.clone())
        .bind(&entity.fee_currency)
        .bind(&entity.provider)
        .bind(&entity.status)
        .bind(&entity.error_message)
        .bind(&entity.metadata)
        .bind(uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn delete(&self, id: &str) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            DatabaseError::new(DatabaseErrorKind::Unknown {
                message: format!("Invalid UUID: {}", e),
            })
        })?;
        let result = sqlx::query("DELETE FROM conversion_audits WHERE id = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::from_sqlx)?;
        Ok(result.rows_affected() > 0)
    }
}

impl TransactionalRepository for ConversionAuditRepository {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
