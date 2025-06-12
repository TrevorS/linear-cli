// ABOUTME: Benchmark for CLI startup time to measure performance optimizations
// ABOUTME: Measures cold start time, help command performance, and CLI initialization

use criterion::{Criterion, criterion_group, criterion_main};
use std::process::Command;
use std::time::Duration;

fn benchmark_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");

    // Set reasonable sample size for process spawning
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("help_command", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "-p", "linear-cli", "--", "--help"])
                .output()
                .expect("Failed to run CLI help command");

            assert!(output.status.success(), "Help command should succeed");
            assert!(!output.stdout.is_empty(), "Help should produce output");
        });
    });

    group.bench_function("version_command", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "-p", "linear-cli", "--", "--version"])
                .output()
                .expect("Failed to run CLI version command");

            assert!(output.status.success(), "Version command should succeed");
        });
    });

    // Test startup with invalid command (should fail fast)
    group.bench_function("invalid_command", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "-p", "linear-cli", "--", "nonexistent-command"])
                .output()
                .expect("Failed to run CLI with invalid command");

            assert!(!output.status.success(), "Invalid command should fail");
        });
    });

    group.finish();
}

fn benchmark_config_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");

    // Benchmark config loading performance
    group.bench_function("config_loading", |b| {
        b.iter(|| {
            let _config = linear_cli::config::Config::load().unwrap_or_default();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_cli_startup, benchmark_config_loading);
criterion_main!(benches);
