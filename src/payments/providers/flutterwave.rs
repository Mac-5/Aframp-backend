use crate::payments::error::{PaymentError, PaymentResult};
use crate::payments::provider::PaymentProvider;
use crate::payments::types::{
    PaymentRequest, PaymentResponse, PaymentState, ProviderName, StatusRequest, StatusResponse,
    WebhookEvent, WebhookVerificationResult, WithdrawalRequest, WithdrawalResponse,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct FlutterwaveConfig {
    pub secret_key: String,
    pub webhook_secret: Option<String>,
}

impl FlutterwaveConfig {
    pub fn from_env() -> PaymentResult<Self> {
        let secret_key = std::env::var("FLUTTERWAVE_SECRET_KEY").unwrap_or_default();
        if secret_key.is_empty() {
            return Err(PaymentError::ValidationError {
                message: "FLUTTERWAVE_SECRET_KEY environment variable is required".to_string(),
                field: Some("FLUTTERWAVE_SECRET_KEY".to_string()),
            });
        }
        Ok(Self {
            secret_key,
            webhook_secret: std::env::var("FLUTTERWAVE_WEBHOOK_SECRET").ok(),
        })
    }
}

pub struct FlutterwaveProvider {
    _config: FlutterwaveConfig,
}

impl FlutterwaveProvider {
    pub fn from_env() -> PaymentResult<Self> {
        Ok(Self {
            _config: FlutterwaveConfig::from_env()?,
        })
    }
}

#[async_trait]
impl PaymentProvider for FlutterwaveProvider {
    async fn initiate_payment(&self, _request: PaymentRequest) -> PaymentResult<PaymentResponse> {
        Err(PaymentError::ProviderError {
            provider: "flutterwave".to_string(),
            message: "not implemented yet".to_string(),
            provider_code: None,
            retryable: false,
        })
    }

    async fn verify_payment(&self, _request: StatusRequest) -> PaymentResult<StatusResponse> {
        Err(PaymentError::ProviderError {
            provider: "flutterwave".to_string(),
            message: "not implemented yet".to_string(),
            provider_code: None,
            retryable: false,
        })
    }

    async fn process_withdrawal(
        &self,
        _request: WithdrawalRequest,
    ) -> PaymentResult<WithdrawalResponse> {
        Err(PaymentError::ProviderError {
            provider: "flutterwave".to_string(),
            message: "not implemented yet".to_string(),
            provider_code: None,
            retryable: false,
        })
    }

    async fn get_payment_status(&self, request: StatusRequest) -> PaymentResult<StatusResponse> {
        self.verify_payment(request).await
    }

    fn name(&self) -> ProviderName {
        ProviderName::Flutterwave
    }

    fn supported_currencies(&self) -> &'static [&'static str] {
        &["NGN", "GHS", "KES", "ZAR"]
    }

    fn supported_countries(&self) -> &'static [&'static str] {
        &["NG", "GH", "KE", "ZA"]
    }

    fn verify_webhook(
        &self,
        _payload: &[u8],
        _signature: &str,
    ) -> PaymentResult<WebhookVerificationResult> {
        Ok(WebhookVerificationResult {
            valid: false,
            reason: Some("not implemented yet".to_string()),
        })
    }

    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent> {
        let parsed = serde_json::from_slice(payload).unwrap_or_else(|_| serde_json::json!({}));
        Ok(WebhookEvent {
            provider: ProviderName::Flutterwave,
            event_type: "unknown".to_string(),
            transaction_reference: None,
            provider_reference: None,
            status: Some(PaymentState::Unknown),
            payload: parsed,
            received_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

