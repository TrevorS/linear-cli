// ABOUTME: Benchmark for parsing operations including config files and API responses
// ABOUTME: Tests TOML config parsing, JSON response parsing, and argument parsing

use criterion::{criterion_group, criterion_main, Criterion};
use linear_cli::config::Config;

const SAMPLE_CONFIG_TOML: &str = r#"
# Default values for commands
default_team = "ENG"
default_assignee = "me"
preferred_format = "table"
api_url = "https://api.linear.app/graphql"

# Command aliases
[aliases]
my = ["issues", "--assignee", "me"]
todo = ["issues", "--status", "todo", "--assignee", "me"]
standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]
review = ["issues", "--status", "in_progress", "--team", "ENG"]
urgent = ["issues", "--priority", "urgent"]

# Shell completion settings
[completions]
cache_duration = "1h"
enable_dynamic = true
"#;

const SAMPLE_JSON_RESPONSE: &str = r#"
{
  "data": {
    "issues": {
      "nodes": [
        {
          "id": "issue-1",
          "number": 1,
          "title": "Fix authentication bug in login flow",
          "description": "Users are experiencing issues when logging in with OAuth. The token refresh mechanism is not working properly.",
          "state": {
            "name": "In Progress"
          },
          "priority": 2,
          "assignee": {
            "id": "user-1",
            "name": "John Doe",
            "email": "john@example.com"
          },
          "team": {
            "id": "team-1",
            "key": "ENG",
            "name": "Engineering"
          },
          "createdAt": "2024-01-01T00:00:00Z",
          "updatedAt": "2024-01-02T00:00:00Z",
          "url": "https://linear.app/test/issue/ENG-1",
          "labels": {
            "nodes": [
              {"name": "bug"},
              {"name": "auth"}
            ]
          }
        },
        {
          "id": "issue-2",
          "number": 2,
          "title": "Implement new search feature",
          "description": "Add full-text search capability to the application with proper indexing and filtering.",
          "state": {
            "name": "Todo"
          },
          "priority": 3,
          "assignee": null,
          "team": {
            "id": "team-1",
            "key": "ENG",
            "name": "Engineering"
          },
          "createdAt": "2024-01-03T00:00:00Z",
          "updatedAt": "2024-01-03T00:00:00Z",
          "url": "https://linear.app/test/issue/ENG-2",
          "labels": {
            "nodes": [
              {"name": "feature"},
              {"name": "search"}
            ]
          }
        }
      ]
    }
  }
}
"#;

fn benchmark_config_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");

    group.bench_function("parse_toml_config", |b| {
        b.iter(|| {
            let config: Config =
                toml::from_str(SAMPLE_CONFIG_TOML).expect("Should parse valid TOML config");
            config
        });
    });

    // Test parsing larger config with more aliases
    let large_config = format!(
        "{}\n{}",
        SAMPLE_CONFIG_TOML,
        (0..50)
            .map(|i| format!(
                "alias{} = [\"issues\", \"--status\", \"todo\", \"--limit\", \"{}\"]",
                i,
                i + 1
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );

    group.bench_function("parse_large_toml_config", |b| {
        b.iter(|| {
            let config: Config =
                toml::from_str(&large_config).expect("Should parse large TOML config");
            config
        });
    });

    group.finish();
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    group.bench_function("parse_api_response", |b| {
        b.iter(|| {
            let parsed: serde_json::Value =
                serde_json::from_str(SAMPLE_JSON_RESPONSE).expect("Should parse JSON response");
            parsed
        });
    });

    // Test parsing larger JSON response
    let large_response = {
        let mut base: serde_json::Value = serde_json::from_str(SAMPLE_JSON_RESPONSE).unwrap();
        let nodes = base["data"]["issues"]["nodes"].as_array().unwrap().clone();

        // Duplicate the issues to create a larger response
        let mut large_nodes = Vec::new();
        for i in 0..100 {
            for node in &nodes {
                let mut new_node = node.clone();
                new_node["id"] = serde_json::Value::String(format!("issue-{}", i * 2 + 1));
                new_node["number"] = serde_json::Value::Number((i * 2 + 1).into());
                new_node["title"] = serde_json::Value::String(format!("Issue {} title", i * 2 + 1));
                large_nodes.push(new_node);
            }
        }

        base["data"]["issues"]["nodes"] = serde_json::Value::Array(large_nodes);
        serde_json::to_string(&base).unwrap()
    };

    group.bench_function("parse_large_api_response", |b| {
        b.iter(|| {
            let parsed: serde_json::Value =
                serde_json::from_str(&large_response).expect("Should parse large JSON response");
            parsed
        });
    });

    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let sample_json = r#"{"id": "issue-1", "title": "Test issue", "status": "Todo"}"#;

    group.bench_function("json_value_parsing", |b| {
        b.iter(|| {
            serde_json::from_str::<serde_json::Value>(sample_json).expect("Should parse JSON value")
        });
    });

    group.bench_function("string_concatenation", |b| {
        b.iter(|| {
            let mut result = String::new();
            for i in 0..100 {
                result.push_str(&format!("item-{} ", i));
            }
            result
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_config_parsing,
    benchmark_json_parsing,
    benchmark_string_operations
);
criterion_main!(benches);
