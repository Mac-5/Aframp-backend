//! Trustline operation service
//! Handles create/update/remove tracking for trustline operations.

use crate::database::error::DatabaseError;
use crate::database::trustline_operation_repository::{
    TrustlineOperation, TrustlineOperationRepository,
};
use uuid::Uuid;

/// Input for creating a trustline operation
#[derive(Debug, Clone)]
pub struct TrustlineOperationInput {
    pub wallet_address: String,
    pub asset_code: String,
    pub issuer: Option<String>,
    pub operation_type: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// Service for trustline operation tracking
pub struct TrustlineOperationService {
    repo: TrustlineOperationRepository,
}

impl TrustlineOperationService {
    pub fn new(repo: TrustlineOperationRepository) -> Self {
        Self { repo }
    }

    /// Record a trustline create operation
    pub async fn record_create(
        &self,
        input: TrustlineOperationInput,
    ) -> Result<TrustlineOperation, DatabaseError> {
        self.repo
            .create_operation(
                &input.wallet_address,
                &input.asset_code,
                input.issuer.as_deref(),
                "create",
                &input.status,
                input.transaction_hash.as_deref(),
                input.error_message.as_deref(),
                input.metadata,
            )
            .await
    }

    /// Record a trustline update operation
    pub async fn record_update(
        &self,
        input: TrustlineOperationInput,
    ) -> Result<TrustlineOperation, DatabaseError> {
        self.repo
            .create_operation(
                &input.wallet_address,
                &input.asset_code,
                input.issuer.as_deref(),
                "update",
                &input.status,
                input.transaction_hash.as_deref(),
                input.error_message.as_deref(),
                input.metadata,
            )
            .await
    }

    /// Record a trustline removal operation
    pub async fn record_remove(
        &self,
        input: TrustlineOperationInput,
    ) -> Result<TrustlineOperation, DatabaseError> {
        self.repo
            .create_operation(
                &input.wallet_address,
                &input.asset_code,
                input.issuer.as_deref(),
                "remove",
                &input.status,
                input.transaction_hash.as_deref(),
                input.error_message.as_deref(),
                input.metadata,
            )
            .await
    }

    /// Update an operation status
    pub async fn update_status(
        &self,
        operation_id: Uuid,
        status: &str,
        transaction_hash: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<TrustlineOperation, DatabaseError> {
        self.repo
            .update_status(operation_id, status, transaction_hash, error_message)
            .await
    }
}
