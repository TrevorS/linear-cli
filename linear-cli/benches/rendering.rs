// ABOUTME: Benchmark for output rendering performance including table formatting
// ABOUTME: Tests issue list rendering, markdown parsing, and syntax highlighting

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use linear_cli::output::{JsonFormatter, OutputFormat, TableFormatter};
use linear_sdk::Issue;

fn create_mock_issues(count: usize) -> Vec<Issue> {
    (0..count)
        .map(|i| Issue {
            id: format!("issue-{}", i + 1),
            identifier: format!("ENG-{}", i + 1),
            title: format!("Test issue number {} with a reasonably long title", i + 1),
            status: match i % 4 {
                0 => "Todo".to_string(),
                1 => "In Progress".to_string(),
                2 => "Done".to_string(),
                _ => "Backlog".to_string(),
            },
            state_id: format!("state-{}", i),
            assignee: if i % 3 == 0 {
                Some(format!("User {}", i + 1))
            } else {
                None
            },
            assignee_id: if i % 3 == 0 {
                Some(format!("user-{}", i))
            } else {
                None
            },
            team: Some(format!("Team {}", i % 3)),
            team_id: format!("team-{}", i % 3),
        })
        .collect()
}

fn benchmark_table_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_rendering");

    for issue_count in [5, 10, 20, 50].iter() {
        let issues = create_mock_issues(*issue_count);

        group.bench_with_input(
            BenchmarkId::new("format_issues_table", issue_count),
            issue_count,
            |b, _| {
                b.iter(|| {
                    let formatter = TableFormatter::new_with_interactive(false, false);
                    formatter.format_issues(&issues).unwrap()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_issues_table_colored", issue_count),
            issue_count,
            |b, _| {
                b.iter(|| {
                    let formatter = TableFormatter::new_with_interactive(true, true);
                    formatter.format_issues(&issues).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_json_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_rendering");

    for issue_count in [5, 10, 20, 50].iter() {
        let issues = create_mock_issues(*issue_count);

        group.bench_with_input(
            BenchmarkId::new("format_issues_json", issue_count),
            issue_count,
            |b, _| {
                b.iter(|| {
                    let formatter = JsonFormatter::new(true);
                    formatter.format_issues(&issues).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn benchmark_markdown_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_rendering");

    let markdown_text = r#"
# Test Issue Description

This is a **bold** test with some *italic* text and `code`.

## Code Block

```rust
fn main() {
    println!("Hello, world!");
}
```

## List Items

- Item 1
- Item 2
- Item 3

## Links

[Linear](https://linear.app) is great!
"#;

    group.bench_function("parse_markdown", |b| {
        b.iter(|| {
            use pulldown_cmark::{Parser, html};

            let parser = Parser::new(markdown_text);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            html_output
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_table_rendering,
    benchmark_json_rendering,
    benchmark_markdown_rendering
);
criterion_main!(benches);
