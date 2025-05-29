// ABOUTME: Build script for generating GraphQL types from schema and queries
// ABOUTME: Processes all .graphql files in graphql/queries directory

use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=graphql/schema.json");
    println!("cargo:rerun-if-changed=graphql/queries");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let queries_dir = Path::new("graphql/queries");

    if queries_dir.exists() {
        for entry in fs::read_dir(queries_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("graphql") {
                let query_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                
                let mut options = graphql_client_codegen::GraphQLClientCodegenOptions::new(
                    graphql_client_codegen::CodegenMode::Cli
                );
                options.set_response_derives("Debug".to_string());
                
                let tokens = graphql_client_codegen::generate_module_token_stream(
                    path,
                    Path::new("graphql/schema.json"),
                    options,
                )
                .unwrap();
                
                let output_path = format!("{}/{}.rs", out_dir, query_name);
                fs::write(&output_path, tokens.to_string()).unwrap();
            }
        }
    }
}