// ABOUTME: xtask binary for build automation and schema management
// ABOUTME: Provides commands to download and update Linear GraphQL schema

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation tasks for linear-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download the latest Linear GraphQL schema
    Schema {
        /// API key for Linear (optional, will use CARGO_PKG_METADATA_XTASK_LINEAR_API_KEY env var if not provided)
        #[arg(long)]
        api_key: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Schema { api_key } => {
            println!("Downloading Linear GraphQL schema...");
            download_schema(api_key)?;
        }
    }

    Ok(())
}

fn download_schema(api_key: Option<String>) -> Result<()> {
    let api_key = api_key
        .or_else(|| std::env::var("CARGO_PKG_METADATA_XTASK_LINEAR_API_KEY").ok())
        .or_else(|| std::env::var("LINEAR_API_KEY").ok())
        .context("No API key provided. Use --api-key or set LINEAR_API_KEY environment variable")?;

    let introspection_query = serde_json::json!({
        "query": r#"
            query IntrospectionQuery {
              __schema {
                queryType { name }
                mutationType { name }
                subscriptionType { name }
                types {
                  ...FullType
                }
                directives {
                  name
                  description
                  locations
                  args {
                    ...InputValue
                  }
                }
              }
            }

            fragment FullType on __Type {
              kind
              name
              description
              fields(includeDeprecated: true) {
                name
                description
                args {
                  ...InputValue
                }
                type {
                  ...TypeRef
                }
                isDeprecated
                deprecationReason
              }
              inputFields {
                ...InputValue
              }
              interfaces {
                ...TypeRef
              }
              enumValues(includeDeprecated: true) {
                name
                description
                isDeprecated
                deprecationReason
              }
              possibleTypes {
                ...TypeRef
              }
            }

            fragment InputValue on __InputValue {
              name
              description
              type { ...TypeRef }
              defaultValue
            }

            fragment TypeRef on __Type {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                  ofType {
                    kind
                    name
                    ofType {
                      kind
                      name
                      ofType {
                        kind
                        name
                        ofType {
                          kind
                          name
                          ofType {
                            kind
                            name
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
        "#
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.linear.app/graphql")
        .header("Authorization", api_key)
        .json(&introspection_query)
        .send()
        .context("Failed to send introspection query")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download schema: {}", response.status());
    }

    let schema_response: serde_json::Value =
        response.json().context("Failed to parse schema response")?;

    // Extract just the schema portion
    let schema = schema_response
        .get("data")
        .context("No data field in response")?;

    // Create the output directory if it doesn't exist
    let output_dir = Path::new("../linear-sdk/graphql");
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    // Write the schema to file
    let output_path = output_dir.join("schema.json");
    let formatted_schema =
        serde_json::to_string_pretty(schema).context("Failed to format schema")?;

    fs::write(&output_path, formatted_schema).context("Failed to write schema file")?;

    println!("Schema downloaded successfully to {:?}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that CLI can be parsed without panicking
        let _ = Cli::try_parse_from(["xtask", "schema", "--api-key", "test"]);
    }
}
