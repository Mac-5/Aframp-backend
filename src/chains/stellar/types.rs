use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarAccountInfo {
    pub account_id: String,
    pub sequence: i64,
    pub subentry_count: u32,
    pub thresholds: Thresholds,
    pub flags: AccountFlags,
    pub balances: Vec<AssetBalance>,
    pub signers: Vec<Signer>,
    pub data: HashMap<String, String>,
    pub last_modified_ledger: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub low_threshold: u8,
    pub med_threshold: u8,
    pub high_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFlags {
    pub auth_required: bool,
    pub auth_revocable: bool,
    pub auth_immutable: bool,
    pub auth_clawback_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
    pub balance: String,
    pub limit: String,
    pub is_authorized: bool,
    pub is_authorized_to_maintain_liabilities: bool,
    pub last_modified_ledger: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    pub key: String,
    pub weight: u8,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonAccount {
    pub _links: HashMap<String, serde_json::Value>,
    pub id: String,
    pub account_id: String,
    pub sequence: String,
    pub subentry_count: u32,
    pub thresholds: Thresholds,
    pub flags: AccountFlags,
    pub balances: Vec<HorizonBalance>,
    pub signers: Vec<Signer>,
    pub data: HashMap<String, String>,
    pub last_modified_ledger: u64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonBalance {
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
    pub balance: String,
    pub limit: String,
    pub is_authorized: bool,
    pub is_authorized_to_maintain_liabilities: bool,
    pub last_modified_ledger: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub horizon_url: String,
    pub response_time_ms: u64,
    pub last_check: String,
    pub error_message: Option<String>,
}

impl From<HorizonAccount> for StellarAccountInfo {
    fn from(account: HorizonAccount) -> Self {
        Self {
            account_id: account.account_id,
            sequence: account.sequence.parse().unwrap_or(0),
            subentry_count: account.subentry_count,
            thresholds: account.thresholds,
            flags: account.flags,
            balances: account
                .balances
                .into_iter()
                .map(AssetBalance::from)
                .collect(),
            signers: account.signers,
            data: account.data,
            last_modified_ledger: account.last_modified_ledger as u32,
            created_at: account.created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        }
    }
}

impl From<HorizonBalance> for AssetBalance {
    fn from(balance: HorizonBalance) -> Self {
        Self {
            asset_type: balance.asset_type,
            asset_code: balance.asset_code,
            asset_issuer: balance.asset_issuer,
            balance: balance.balance,
            limit: balance.limit,
            is_authorized: balance.is_authorized,
            is_authorized_to_maintain_liabilities: balance.is_authorized_to_maintain_liabilities,
            last_modified_ledger: balance.last_modified_ledger.map(|v| v as u32),
        }
    }
}

pub fn is_valid_stellar_address(address: &str) -> bool {
    // Basic validation: Stellar addresses are 56 characters starting with 'G'
    address.len() == 56 && address.starts_with('G') && address.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn extract_afri_balance(balances: &[AssetBalance]) -> Option<String> {
    balances
        .iter()
        .find(|balance| {
            balance.asset_type == "credit_alphanum4"
                && balance.asset_code.as_ref().map_or(false, |code| code == "AFRI")
        })
        .map(|balance| balance.balance.clone())
}
