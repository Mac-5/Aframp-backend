use crate::database::error::{DatabaseError, DatabaseErrorKind};
use crate::database::repository::{Repository, TransactionalRepository};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Trustline operation entity
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TrustlineOperation {
    pub id: Uuid,
    pub wallet_address: String,
    pub asset_code: String,
    pub issuer: Option<String>,
    pub operation_type: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Repository for trustline operations tracking
pub struct TrustlineOperationRepository {
    pool: PgPool,
}

impl TrustlineOperationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a trustline operation record
    pub async fn create_operation(
        &self,
        wallet_address: &str,
        asset_code: &str,
        issuer: Option<&str>,
        operation_type: &str,
        status: &str,
        transaction_hash: Option<&str>,
        error_message: Option<&str>,
        metadata: serde_json::Value,
    ) -> Result<TrustlineOperation, DatabaseError> {
        sqlx::query_as::<_, TrustlineOperation>(
            "INSERT INTO trustline_operations 
             (wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at",
        )
        .bind(wallet_address)
        .bind(asset_code)
        .bind(issuer)
        .bind(operation_type)
        .bind(status)
        .bind(transaction_hash)
        .bind(error_message)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    /// Update operation status
    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        transaction_hash: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<TrustlineOperation, DatabaseError> {
        sqlx::query_as::<_, TrustlineOperation>(
            "UPDATE trustline_operations 
             SET status = $2, transaction_hash = $3, error_message = $4, updated_at = NOW()
             WHERE id = $1 
             RETURNING id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at",
        )
        .bind(id)
        .bind(status)
        .bind(transaction_hash)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    /// Find latest operations for a wallet
    pub async fn find_by_wallet(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<TrustlineOperation>, DatabaseError> {
        sqlx::query_as::<_, TrustlineOperation>(
            "SELECT id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at 
             FROM trustline_operations 
             WHERE wallet_address = $1 
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }
}

#[async_trait]
impl Repository for TrustlineOperationRepository {
    type Entity = TrustlineOperation;

    async fn find_by_id(&self, id: &str) -> Result<Option<Self::Entity>, DatabaseError> {
        let uuid = Uuid::parse_str(id).map_err(|e| {
            DatabaseError::new(DatabaseErrorKind::Unknown {
                message: format!("Invalid UUID: {}", e),
            })
        })?;
        sqlx::query_as::<_, TrustlineOperation>(
            "SELECT id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at 
             FROM trustline_operations WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn find_all(&self) -> Result<Vec<Self::Entity>, DatabaseError> {
        sqlx::query_as::<_, TrustlineOperation>(
            "SELECT id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at 
             FROM trustline_operations ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from_sqlx)
    }

    async fn insert(&self, entity: &Self::Entity) -> Result<Self::Entity, DatabaseError> {
        sqlx::query_as::<_, TrustlineOperation>(
            "INSERT INTO trustline_operations 
             (id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) 
             RETURNING id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at",
        )
        .bind(entity.id)
        .bind(&entity.wallet_address)
        .bind(&entity.asset_code)
        .bind(&entity.issuer)
        .bind(&entity.operation_type)
        .bind(&entity.status)
        .bind(&entity.transaction_hash)
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
        sqlx::query_as::<_, TrustlineOperation>(
            "UPDATE trustline_operations 
             SET wallet_address = $1, asset_code = $2, issuer = $3, operation_type = $4, status = $5, transaction_hash = $6, error_message = $7, metadata = $8, updated_at = NOW()
             WHERE id = $9 
             RETURNING id, wallet_address, asset_code, issuer, operation_type, status, transaction_hash, error_message, metadata, created_at, updated_at",
        )
        .bind(&entity.wallet_address)
        .bind(&entity.asset_code)
        .bind(&entity.issuer)
        .bind(&entity.operation_type)
        .bind(&entity.status)
        .bind(&entity.transaction_hash)
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
        let result = sqlx::query("DELETE FROM trustline_operations WHERE id = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::from_sqlx)?;
        Ok(result.rows_affected() > 0)
    }
}

impl TransactionalRepository for TrustlineOperationRepository {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
