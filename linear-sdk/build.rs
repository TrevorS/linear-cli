// ABOUTME: Build script for generating GraphQL types from schema and queries
// ABOUTME: Processes all .graphql files in graphql/queries directory

use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=graphql/schema.json");
    println!("cargo:rerun-if-changed=graphql/queries");
    println!("cargo:rerun-if-changed=graphql/mutations");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Process queries
    let queries_dir = Path::new("graphql/queries");
    if queries_dir.exists() {
        process_graphql_dir(queries_dir, &out_dir);
    }

    // Process mutations
    let mutations_dir = Path::new("graphql/mutations");
    if mutations_dir.exists() {
        process_graphql_dir(mutations_dir, &out_dir);
    }
}

fn process_graphql_dir(dir: &Path, out_dir: &str) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("graphql") {
            let query_name = path.file_stem().unwrap().to_str().unwrap().to_string();

            let mut options = graphql_client_codegen::GraphQLClientCodegenOptions::new(
                graphql_client_codegen::CodegenMode::Cli,
            );
            options.set_response_derives("Debug".to_string());

            let tokens = graphql_client_codegen::generate_module_token_stream(
                path,
                Path::new("graphql/schema.json"),
                options,
            )
            .unwrap();

            let output_path = format!("{out_dir}/{query_name}.rs");
            fs::write(&output_path, tokens.to_string()).unwrap();
        }
    }
}
