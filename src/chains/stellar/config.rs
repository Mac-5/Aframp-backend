use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StellarNetwork {
    Testnet,
    Mainnet,
}

#[allow(dead_code)]
impl StellarNetwork {
    pub fn horizon_url(&self) -> &'static str {
        match self {
            StellarNetwork::Testnet => "https://horizon-testnet.stellar.org",
            StellarNetwork::Mainnet => "https://horizon.stellar.org",
        }
    }

    pub fn network_passphrase(&self) -> &'static str {
        match self {
            StellarNetwork::Testnet => "Test SDF Network ; September 2015",
            StellarNetwork::Mainnet => "Public Global Stellar Network ; September 2015",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarConfig {
    pub network: StellarNetwork,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub health_check_interval: Duration,
}

impl Default for StellarConfig {
    fn default() -> Self {
        Self {
            network: StellarNetwork::Testnet,
            request_timeout: Duration::from_secs(15),
            max_retries: 3,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

impl StellarConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let network = match std::env::var("STELLAR_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string())
            .to_lowercase()
            .as_str()
        {
            "mainnet" => {
                info!("Initializing Stellar client for MAINNET");
                StellarNetwork::Mainnet
            }
            "testnet" => {
                info!("Initializing Stellar client for TESTNET");
                StellarNetwork::Testnet
            }
            other => {
                warn!("Invalid STELLAR_NETWORK '{}', defaulting to testnet", other);
                StellarNetwork::Testnet
            }
        };

        let request_timeout = std::env::var("STELLAR_REQUEST_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| {
                info!("Using default request timeout: 15 seconds");
                Duration::from_secs(15)
            });

        let max_retries = std::env::var("STELLAR_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let health_check_interval = std::env::var("STELLAR_HEALTH_CHECK_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| {
                info!("Using default health check interval: 30 seconds");
                Duration::from_secs(30)
            });

        Ok(Self {
            network,
            request_timeout,
            max_retries,
            health_check_interval,
        })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.request_timeout.as_secs() == 0 {
            anyhow::bail!("Request timeout must be greater than 0");
        }

        if self.max_retries == 0 {
            anyhow::bail!("Max retries must be greater than 0");
        }

        if self.health_check_interval.as_secs() == 0 {
            anyhow::bail!("Health check interval must be greater than 0");
        }

        info!(
            "Stellar configuration validated - Network: {:?}, Timeout: {:?}, Max retries: {}",
            self.network, self.request_timeout, self.max_retries
        );

        Ok(())
    }
}
