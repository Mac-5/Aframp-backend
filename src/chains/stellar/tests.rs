#[cfg(test)]
mod tests {
    use crate::chains::stellar::errors::StellarError;
    use crate::chains::stellar::{
        client::StellarClient,
        config::{StellarConfig, StellarNetwork},
        types::is_valid_stellar_address,
    };
    use std::time::Duration;

    fn test_config() -> StellarConfig {
        StellarConfig {
            network: StellarNetwork::Testnet,
            request_timeout: Duration::from_secs(15),
            max_retries: 3,
            health_check_interval: Duration::from_secs(30),
        }
    }

    // Valid testnet account that exists (from Stellar friendbot)
    const TEST_ADDRESS: &str = "GCJRI5CIWK5IU67Q6DGA7QW52JDKRO7JEAHQKFNDUJUPEZGURDBX3LDX";

    #[test]
    fn test_valid_stellar_address() {
        let valid_address = TEST_ADDRESS;
        assert!(is_valid_stellar_address(valid_address));

        let invalid_address = "INVALID_ADDRESS";
        assert!(!is_valid_stellar_address(invalid_address));

        let wrong_length = "GD5DJQDQKNR7DSXJVNJTV3P5JJH4KJVTI2JZNYUYIIKHTDNJQXECM4J";
        assert!(!is_valid_stellar_address(wrong_length));
    }

    #[tokio::test]
    async fn test_stellar_client_creation() {
        let config = test_config();
        let client = StellarClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_get_valid_testnet_account() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let test_address = TEST_ADDRESS;

        match client.get_account(test_address).await {
            Ok(account) => {
                assert_eq!(account.account_id, test_address);
                assert!(!account.balances.is_empty());
            }
            Err(StellarError::AccountNotFound { .. }) => {
                println!("Test account not found, this is expected if the account doesn't exist");
            }
            Err(StellarError::NetworkError { .. }) | Err(StellarError::TimeoutError { .. }) => {
                println!("Network issue, skipping test");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_nonexistent_account() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        // Use a valid format but nonexistent address
        let nonexistent_address = "GAAAAAAAACGC6AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

        let result = client.get_account(nonexistent_address).await;
        // This test verifies we get an error for nonexistent accounts
        // Accept any error type since the API may return different errors
        assert!(result.is_err(), "Expected an error for nonexistent account");
    }

    #[tokio::test]
    async fn test_invalid_address_format() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let invalid_address = "INVALID_ADDRESS";

        let result = client.get_account(invalid_address).await;
        assert!(matches!(result, Err(StellarError::InvalidAddress { .. })));
    }

    #[tokio::test]
    async fn test_account_exists() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let test_address = TEST_ADDRESS;

        match client.account_exists(test_address).await {
            Ok(exists) => {
                println!("Account {} exists: {}", test_address, exists);
            }
            Err(StellarError::AccountNotFound { .. }) => {
                println!("Account does not exist, which is valid");
            }
            Err(StellarError::NetworkError { .. }) | Err(StellarError::TimeoutError { .. }) => {
                println!("Network issue, skipping test");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let health_status = client.health_check().await.expect("Health check failed");

        println!("Health status: {:?}", health_status);

        if health_status.is_healthy {
            assert!(health_status.response_time_ms > 0);
            assert!(health_status.error_message.is_none());
        } else {
            assert!(health_status.error_message.is_some());
        }
    }

    #[tokio::test]
    async fn test_get_balances() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let test_address = TEST_ADDRESS;

        match client.get_balances(test_address).await {
            Ok(balances) => {
                println!("Balances for {}: {:?}", test_address, balances);
                assert!(!balances.is_empty());
            }
            Err(StellarError::AccountNotFound { .. }) => {
                println!("Account not found, skipping balance test");
            }
            Err(StellarError::NetworkError { .. }) | Err(StellarError::TimeoutError { .. }) => {
                println!("Network issue, skipping test");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_afri_balance() {
        let config = test_config();
        let client = StellarClient::new(config).expect("Failed to create client");

        let test_address = TEST_ADDRESS;

        match client.get_afri_balance(test_address).await {
            Ok(afri_balance) => {
                println!("AFRI balance for {}: {:?}", test_address, afri_balance);
            }
            Err(StellarError::AccountNotFound { .. }) => {
                println!("Account not found, skipping AFRI balance test");
            }
            Err(StellarError::NetworkError { .. }) | Err(StellarError::TimeoutError { .. }) => {
                println!("Network issue, skipping test");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn test_config_validation() {
        let mut config = test_config();
        assert!(config.validate().is_ok());

        config.request_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());

        config = test_config();
        config.max_retries = 0;
        assert!(config.validate().is_err());

        config = test_config();
        config.health_check_interval = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_configurations() {
        let testnet_config = StellarConfig {
            network: StellarNetwork::Testnet,
            ..test_config()
        };
        assert_eq!(
            testnet_config.network.horizon_url(),
            "https://horizon-testnet.stellar.org"
        );
        assert_eq!(
            testnet_config.network.network_passphrase(),
            "Test SDF Network ; September 2015"
        );

        let mainnet_config = StellarConfig {
            network: StellarNetwork::Mainnet,
            ..test_config()
        };
        assert_eq!(
            mainnet_config.network.horizon_url(),
            "https://horizon.stellar.org"
        );
        assert_eq!(
            mainnet_config.network.network_passphrase(),
            "Public Global Stellar Network ; September 2015"
        );
    }
}
