// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use anyhow::Result;
use linear_sdk::LinearClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let api_key = match env::var("LINEAR_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: No LINEAR_API_KEY environment variable found");
            eprintln!();
            eprintln!("Please set your Linear API key:");
            eprintln!("export LINEAR_API_KEY=lin_api_xxxxx");
            eprintln!();
            eprintln!("Get your API key from: https://linear.app/settings/api");
            std::process::exit(1);
        }
    };

    let client = LinearClient::new(api_key)?;
    let viewer_data = client.execute_viewer_query().await?;

    println!("Viewer Information:");
    println!("  Name: {}", viewer_data.viewer.name);
    println!("  Email: {}", viewer_data.viewer.email);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Placeholder test until we have actual functionality
        assert_eq!(1 + 1, 2);
    }
}
