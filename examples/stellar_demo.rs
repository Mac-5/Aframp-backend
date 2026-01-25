use Bitmesh_backend::chains::stellar::{
    client::StellarClient,
    config::{StellarConfig, StellarNetwork},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = StellarConfig {
        network: StellarNetwork::Testnet,
        request_timeout: std::time::Duration::from_secs(10),
        max_retries: 3,
        health_check_interval: std::time::Duration::from_secs(30),
    };

    let client = StellarClient::new(config)?;

    println!("=== Stellar Client Demo ===");

    // Health check
    println!("Performing health check...");
    let health = client.health_check().await?;
    println!("Health status: {:?}", health);

    // Test with a known testnet address
    let test_address = "GD5DJQDQKNR7DSXJVNJTV3P5JJH4KJVTI2JZNYUYIIKHTDNJQXECM4JQ";
    println!("\nTesting account: {}", test_address);

    // Check if account exists
    match client.account_exists(test_address).await {
        Ok(exists) => println!("Account exists: {}", exists),
        Err(e) => println!("Error checking account: {}", e),
    }

    // Get account details
    match client.get_account(test_address).await {
        Ok(account) => {
            println!("Account ID: {}", account.account_id);
            println!("Sequence: {}", account.sequence);
            println!("Balances:");
            for balance in &account.balances {
                println!("  {}: {}", balance.asset_type, balance.balance);
            }
        }
        Err(e) => println!("Error getting account: {}", e),
    }

    Ok(())
}
