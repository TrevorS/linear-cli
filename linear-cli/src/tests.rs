use crate::*;
use clap::CommandFactory;
use serial_test::serial;

#[test]
fn test_cli_structure() {
    let cli = Cli::command();

    // Verify basic structure
    assert_eq!(cli.get_name(), "linear");

    // Check that we have the issues subcommand
    let issues_cmd = cli
        .find_subcommand("issues")
        .expect("issues command should exist");
    assert_eq!(issues_cmd.get_name(), "issues");

    // Check that we have the issue subcommand
    let issue_cmd = cli
        .find_subcommand("issue")
        .expect("issue command should exist");
    assert_eq!(issue_cmd.get_name(), "issue");

    // Check the limit argument
    let limit_arg = issues_cmd
        .get_arguments()
        .find(|arg| arg.get_id() == "limit")
        .expect("limit argument should exist");
    assert!(!limit_arg.is_required_set());

    // Check the id argument for issue command
    let id_arg = issue_cmd
        .get_arguments()
        .find(|arg| arg.get_id() == "id")
        .expect("id argument should exist");
    assert!(id_arg.is_required_set());
}

#[test]
fn test_parse_issues_command() {
    use clap::Parser;

    // Test default limit
    let cli = Cli::try_parse_from(["linear", "issues"]).unwrap();
    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee: _,
            status: _,
            team: _,
        } => {
            assert_eq!(limit, 20);
            assert!(!json);
            assert!(!pretty);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test custom limit
    let cli = Cli::try_parse_from(["linear", "issues", "--limit", "5"]).unwrap();
    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee: _,
            status: _,
            team: _,
        } => {
            assert_eq!(limit, 5);
            assert!(!json);
            assert!(!pretty);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test short form
    let cli = Cli::try_parse_from(["linear", "issues", "-l", "10"]).unwrap();
    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee: _,
            status: _,
            team: _,
        } => {
            assert_eq!(limit, 10);
            assert!(!json);
            assert!(!pretty);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test JSON flag
    let cli = Cli::try_parse_from(["linear", "issues", "--json"]).unwrap();
    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee: _,
            status: _,
            team: _,
        } => {
            assert_eq!(limit, 20);
            assert!(json);
            assert!(!pretty);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test JSON with pretty flag
    let cli = Cli::try_parse_from(["linear", "issues", "--json", "--pretty"]).unwrap();
    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee: _,
            status: _,
            team: _,
        } => {
            assert_eq!(limit, 20);
            assert!(json);
            assert!(pretty);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test pretty flag requires json (should fail)
    let result = Cli::try_parse_from(["linear", "issues", "--pretty"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_issue_command() {
    use clap::Parser;

    // Test basic issue command
    let cli = Cli::try_parse_from(["linear", "issue", "ENG-123"]).unwrap();
    match cli.command {
        Commands::Issue { id, json, raw, .. } => {
            assert_eq!(id, "ENG-123");
            assert!(!json);
            assert!(!raw);
        }
        #[cfg(feature = "oauth")]
        Commands::Projects { .. } => panic!("Expected Issue command"),
        Commands::Teams { .. } => panic!("Expected Issue command"),
        Commands::Comments { .. } => panic!("Expected Issue command"),
        Commands::MyWork { .. } => panic!("Expected Issue command"),
        Commands::Search { .. } => panic!("Expected Issue command"),
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issue command")
        }
        _ => panic!("Expected Issue command"),
    }

    // Test issue command with JSON
    let cli = Cli::try_parse_from(["linear", "issue", "ENG-456", "--json"]).unwrap();
    match cli.command {
        Commands::Issue { id, json, raw, .. } => {
            assert_eq!(id, "ENG-456");
            assert!(json);
            assert!(!raw);
        }
        #[cfg(feature = "oauth")]
        Commands::Projects { .. } => panic!("Expected Issue command"),
        Commands::Teams { .. } => panic!("Expected Issue command"),
        Commands::Comments { .. } => panic!("Expected Issue command"),
        Commands::MyWork { .. } => panic!("Expected Issue command"),
        Commands::Search { .. } => panic!("Expected Issue command"),
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issue command")
        }
        _ => panic!("Expected Issue command"),
    }

    // Test issue command with UUID
    let cli = Cli::try_parse_from(["linear", "issue", "abc-123-def-456"]).unwrap();
    match cli.command {
        Commands::Issue { id, json, raw, .. } => {
            assert_eq!(id, "abc-123-def-456");
            assert!(!json);
            assert!(!raw);
        }
        #[cfg(feature = "oauth")]
        Commands::Projects { .. } => panic!("Expected Issue command"),
        Commands::Teams { .. } => panic!("Expected Issue command"),
        Commands::Comments { .. } => panic!("Expected Issue command"),
        Commands::MyWork { .. } => panic!("Expected Issue command"),
        Commands::Search { .. } => panic!("Expected Issue command"),
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issue command")
        }
        _ => panic!("Expected Issue command"),
    }

    // Test issue command with --raw flag
    let cli = Cli::try_parse_from(["linear", "issue", "ENG-789", "--raw"]).unwrap();
    match cli.command {
        Commands::Issue { id, json, raw, .. } => {
            assert_eq!(id, "ENG-789");
            assert!(!json);
            assert!(raw);
        }
        #[cfg(feature = "oauth")]
        Commands::Projects { .. } => panic!("Expected Issue command"),
        Commands::Teams { .. } => panic!("Expected Issue command"),
        Commands::Comments { .. } => panic!("Expected Issue command"),
        Commands::MyWork { .. } => panic!("Expected Issue command"),
        Commands::Search { .. } => panic!("Expected Issue command"),
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issue command")
        }
        _ => panic!("Expected Issue command"),
    }

    // Test issue command with both --json and --raw flags
    let cli = Cli::try_parse_from(["linear", "issue", "ENG-999", "--json", "--raw"]).unwrap();
    match cli.command {
        Commands::Issue { id, json, raw, .. } => {
            assert_eq!(id, "ENG-999");
            assert!(json);
            assert!(raw);
        }
        #[cfg(feature = "oauth")]
        Commands::Projects { .. } => panic!("Expected Issue command"),
        Commands::Teams { .. } => panic!("Expected Issue command"),
        Commands::Comments { .. } => panic!("Expected Issue command"),
        Commands::MyWork { .. } => panic!("Expected Issue command"),
        Commands::Search { .. } => panic!("Expected Issue command"),
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issue command")
        }
        _ => panic!("Expected Issue command"),
    }

    // Test issue command without ID (should fail)
    let result = Cli::try_parse_from(["linear", "issue"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_filter_arguments() {
    use clap::Parser;

    // Test assignee filter
    let cli = Cli::try_parse_from(["linear", "issues", "--assignee", "me"]).unwrap();
    match cli.command {
        Commands::Issues {
            assignee,
            status,
            team,
            ..
        } => {
            assert_eq!(assignee, Some("me".to_string()));
            assert_eq!(status, None);
            assert_eq!(team, None);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test status filter
    let cli = Cli::try_parse_from(["linear", "issues", "--status", "done"]).unwrap();
    match cli.command {
        Commands::Issues {
            assignee,
            status,
            team,
            ..
        } => {
            assert_eq!(assignee, None);
            assert_eq!(status, Some("done".to_string()));
            assert_eq!(team, None);
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test team filter
    let cli = Cli::try_parse_from(["linear", "issues", "--team", "ENG"]).unwrap();
    match cli.command {
        Commands::Issues {
            assignee,
            status,
            team,
            ..
        } => {
            assert_eq!(assignee, None);
            assert_eq!(status, None);
            assert_eq!(team, Some("ENG".to_string()));
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }

    // Test combined filters
    let cli = Cli::try_parse_from([
        "linear",
        "issues",
        "--assignee",
        "me",
        "--status",
        "in progress",
        "--team",
        "ENG",
    ])
    .unwrap();
    match cli.command {
        Commands::Issues {
            assignee,
            status,
            team,
            ..
        } => {
            assert_eq!(assignee, Some("me".to_string()));
            assert_eq!(status, Some("in progress".to_string()));
            assert_eq!(team, Some("ENG".to_string()));
        }
        Commands::Issue { .. } => panic!("Expected Issues command"),
        Commands::Create { .. } => panic!("Expected Issues command"),
        Commands::Update { .. } => panic!("Expected Issues command"),
        Commands::Close { .. } => panic!("Expected Issues command"),
        Commands::Reopen { .. } => panic!("Expected Issues command"),
        Commands::Comment { .. } => panic!("Expected Issues command"),
        Commands::Attach { .. } => panic!("Expected Issues command"),
        Commands::Relate { .. } => panic!("Expected Issues command"),
        Commands::Projects { .. } => panic!("Expected Issues command"),
        Commands::Teams { .. } => panic!("Expected Issues command"),
        Commands::Comments { .. } => panic!("Expected Issues command"),
        Commands::MyWork { .. } => panic!("Expected Issues command"),
        Commands::Search { .. } => panic!("Expected Issues command"),
        Commands::Status { .. } => panic!("Expected Issues command"),
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
            panic!("Expected Issues command")
        }
    }
}

#[test]
fn test_version_command() {
    use clap::Parser;
    use std::process::Command;

    // Test that version command parses correctly
    let result = Cli::try_parse_from(["linear", "--version"]);
    // This will fail because --version is handled by clap before we get the result
    // But we can test that it doesn't conflict with other parsing
    assert!(result.is_err()); // clap exits early for --version

    // Test that version is included in help output
    let output = Command::new("cargo")
        .args(["run", "-p", "linear-cli", "--", "--help"])
        .output()
        .unwrap();
    let help_output = std::str::from_utf8(&output.stdout).unwrap();
    assert!(help_output.contains("--version") || help_output.contains("-V"));
}

#[test]
fn test_deferred_authentication() {
    use clap::Parser;

    // Test that help commands can be parsed without requiring authentication
    // (Authentication is only checked at runtime when making API calls)

    // Help command should parse successfully
    let cli = Cli::try_parse_from(["linear", "--help"]);
    assert!(cli.is_err()); // clap exits early for --help

    // Subcommand help should parse successfully
    let cli = Cli::try_parse_from(["linear", "issues", "--help"]);
    assert!(cli.is_err()); // clap exits early for --help

    // Regular commands should parse successfully (auth checked later)
    let cli = Cli::try_parse_from(["linear", "issues"]).unwrap();
    match cli.command {
        Commands::Issues { .. } => {} // Success - parsing works without auth
        _ => panic!("Expected Issues command"),
    }

    let cli = Cli::try_parse_from(["linear", "status"]).unwrap();
    match cli.command {
        Commands::Status { .. } => {} // Success - parsing works without auth
        _ => panic!("Expected Status command"),
    }

    let cli = Cli::try_parse_from(["linear", "create", "--title", "Test"]).unwrap();
    match cli.command {
        Commands::Create { .. } => {} // Success - parsing works without auth
        _ => panic!("Expected Create command"),
    }
}

#[test]
#[serial]
fn test_determine_use_color_with_tty() {
    // Save original env vars
    let original_no_color = std::env::var("NO_COLOR").ok();
    let original_term = std::env::var("TERM").ok();

    unsafe {
        // Clear env vars to test default behavior
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("TERM");

        // Mock a TTY scenario (we can't directly mock IsTerminal)
        // but we can test the logic separately
        let use_color = determine_use_color(false, false, true);
        assert!(use_color);

        // Test with no-color flag
        let use_color = determine_use_color(true, false, true);
        assert!(!use_color);

        // Test with NO_COLOR env var
        std::env::set_var("NO_COLOR", "1");
        let use_color = determine_use_color(false, false, true);
        assert!(!use_color);
        std::env::remove_var("NO_COLOR");

        // Test with TERM=dumb
        std::env::set_var("TERM", "dumb");
        let use_color = determine_use_color(false, false, true);
        assert!(!use_color);

        // Test non-TTY (piped/redirected)
        std::env::remove_var("TERM");
        let use_color = determine_use_color(false, false, false);
        assert!(!use_color);

        // Test force-color overrides non-TTY
        let use_color = determine_use_color(false, true, false);
        assert!(use_color);

        // Restore original env vars
        if let Some(val) = original_no_color {
            std::env::set_var("NO_COLOR", val);
        } else {
            std::env::remove_var("NO_COLOR");
        }
        if let Some(val) = original_term {
            std::env::set_var("TERM", val);
        } else {
            std::env::remove_var("TERM");
        }
    }
}

#[test]
#[serial]
fn test_determine_use_color_priority() {
    // Test that flags take precedence over env vars and TTY detection
    let original_no_color = std::env::var("NO_COLOR").ok();
    let original_term = std::env::var("TERM").ok();

    unsafe {
        // Test flag overrides everything
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("TERM");
        let use_color = determine_use_color(true, false, true);
        assert!(!use_color, "--no-color flag should override TTY detection");

        // Test NO_COLOR env var overrides TTY
        std::env::set_var("NO_COLOR", "1");
        let use_color = determine_use_color(false, false, true);
        assert!(!use_color, "NO_COLOR env should override TTY detection");

        // Test TERM=dumb overrides TTY
        std::env::remove_var("NO_COLOR");
        std::env::set_var("TERM", "dumb");
        let use_color = determine_use_color(false, false, true);
        assert!(!use_color, "TERM=dumb should override TTY detection");

        // Test force-color overrides everything
        std::env::set_var("NO_COLOR", "1");
        std::env::set_var("TERM", "dumb");
        let use_color = determine_use_color(false, true, false);
        assert!(use_color, "--force-color should override everything");

        // Restore env vars
        if let Some(val) = original_no_color {
            std::env::set_var("NO_COLOR", val);
        } else {
            std::env::remove_var("NO_COLOR");
        }
        if let Some(val) = original_term {
            std::env::set_var("TERM", val);
        } else {
            std::env::remove_var("TERM");
        }
    }
}

#[test]
fn test_force_color_flag() {
    use clap::Parser;

    // Test force-color flag
    let cli = Cli::try_parse_from(["linear", "--force-color", "issues"]).unwrap();
    assert!(cli.force_color);
    assert!(!cli.no_color);

    // Test that force-color and no-color conflict
    let result = Cli::try_parse_from(["linear", "--force-color", "--no-color", "issues"]);
    assert!(result.is_err());

    // Test no-color flag still works
    let cli = Cli::try_parse_from(["linear", "--no-color", "issues"]).unwrap();
    assert!(!cli.force_color);
    assert!(cli.no_color);
}

#[test]
fn test_limit_validation() {
    use clap::Parser;

    // Test that limit must be at least 1
    let result = Cli::try_parse_from(["linear", "issues", "--limit", "0"]);
    assert!(result.is_err());

    // Test that negative limits are rejected
    let result = Cli::try_parse_from(["linear", "issues", "--limit=-5"]);
    assert!(result.is_err());

    // Test that valid limits work
    let cli = Cli::try_parse_from(["linear", "issues", "--limit", "1"]).unwrap();
    match cli.command {
        Commands::Issues { limit, .. } => {
            assert_eq!(limit, 1);
        }
        _ => panic!("Expected Issues command"),
    }
}

#[test]
fn test_json_output_can_be_parsed() {
    use crate::output::{JsonFormatter, OutputFormat};
    use linear_sdk::Issue;

    // Create test issues
    let issues = vec![
        Issue {
            id: "1".to_string(),
            identifier: "ENG-123".to_string(),
            title: "Test issue".to_string(),
            status: DEFAULT_TODO_STATE.to_string(),
            state_id: "state-todo-123".to_string(),
            assignee: Some("Alice".to_string()),
            assignee_id: Some("user-1".to_string()),
            team: Some("ENG".to_string()),
            team_id: "team-eng-123".to_string(),
        },
        Issue {
            id: "2".to_string(),
            identifier: "ENG-124".to_string(),
            title: "Another issue".to_string(),
            status: DEFAULT_DONE_STATE.to_string(),
            state_id: "state-done-124".to_string(),
            assignee: None,
            assignee_id: None,
            team: Some("ENG".to_string()),
            team_id: "team-eng-124".to_string(),
        },
    ];

    // Test compact JSON
    let formatter = JsonFormatter::new(false);
    let output = formatter.format_issues(&issues).unwrap();

    // Verify it can be parsed back
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["identifier"], "ENG-123");
    assert_eq!(parsed[1]["identifier"], "ENG-124");

    // Test pretty JSON
    let formatter = JsonFormatter::new(true);
    let output = formatter.format_issues(&issues).unwrap();

    // Verify pretty JSON can also be parsed back
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.len(), 2);

    // Verify it's actually pretty printed (contains newlines)
    assert!(output.contains('\n'));
}

// CREATE COMMAND TESTS - Testing CLI parsing and validation

#[test]
fn test_parse_create_command_minimal() {
    use clap::Parser;

    // Test minimal create command with just title and team
    let cli = Cli::try_parse_from(["linear", "create", "--title", "Test Issue", "--team", "ENG"])
        .unwrap();

    match cli.command {
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            open,
            dry_run,
            ..
        } => {
            assert_eq!(title, Some("Test Issue".to_string()));
            assert_eq!(description, None);
            assert_eq!(team, Some("ENG".to_string()));
            assert_eq!(assignee, None);
            assert_eq!(priority, None);
            assert!(!open);
            assert!(!dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_all_fields() {
    use clap::Parser;

    // Test create command with all possible arguments
    let cli = Cli::try_parse_from([
        "linear",
        "create",
        "--title",
        "Complete Test Issue",
        "--description",
        "A complete test description",
        "--team",
        "DESIGN",
        "--assignee",
        "me",
        "--priority",
        "2",
        "--open",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            open,
            dry_run,
            ..
        } => {
            assert_eq!(title, Some("Complete Test Issue".to_string()));
            assert_eq!(description, Some("A complete test description".to_string()));
            assert_eq!(team, Some("DESIGN".to_string()));
            assert_eq!(assignee, Some("me".to_string()));
            assert_eq!(priority, Some(2));
            assert!(open);
            assert!(dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_short_flags() {
    use clap::Parser;

    // Test create command with short flag aliases where available
    let cli = Cli::try_parse_from([
        "linear",
        "create",
        "--title",
        "Short Flag Test",
        "--team",
        "ENG",
        "--priority",
        "1",
    ])
    .unwrap();

    match cli.command {
        Commands::Create {
            title,
            description: _,
            team,
            assignee: _,
            priority,
            open: _,
            dry_run: _,
            ..
        } => {
            assert_eq!(title, Some("Short Flag Test".to_string()));
            assert_eq!(team, Some("ENG".to_string()));
            assert_eq!(priority, Some(1));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_interactive_mode() {
    use clap::Parser;

    // Test create command without any arguments (should trigger interactive mode)
    let cli = Cli::try_parse_from(["linear", "create"]).unwrap();

    match cli.command {
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            open,
            dry_run,
            ..
        } => {
            assert_eq!(title, None);
            assert_eq!(description, None);
            assert_eq!(team, None);
            assert_eq!(assignee, None);
            assert_eq!(priority, None);
            assert!(!open);
            assert!(!dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_priority_validation() {
    use clap::Parser;

    // Test valid priority values
    for priority in 1..=4 {
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Priority Test",
            "--team",
            "ENG",
            "--priority",
            &priority.to_string(),
        ])
        .unwrap();

        match cli.command {
            Commands::Create { priority: p, .. } => {
                assert_eq!(p, Some(priority));
            }
            _ => panic!("Expected Create command"),
        }
    }
}

#[test]
fn test_parse_create_command_invalid_priority() {
    use clap::Parser;

    // Test invalid priority values (should fail parsing)
    for invalid_priority in [0, 5, 10] {
        let result = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Priority Test",
            "--team",
            "ENG",
            "--priority",
            &invalid_priority.to_string(),
        ]);

        assert!(
            result.is_err(),
            "Priority {invalid_priority} should be invalid"
        );
    }
}

#[test]
fn test_parse_create_command_special_assignees() {
    use clap::Parser;

    // Test special assignee values
    let special_assignees = ["me", "unassigned"];

    for assignee in &special_assignees {
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Assignee Test",
            "--team",
            "ENG",
            "--assignee",
            assignee,
        ])
        .unwrap();

        match cli.command {
            Commands::Create { assignee: a, .. } => {
                assert_eq!(a, Some(assignee.to_string()));
            }
            _ => panic!("Expected Create command"),
        }
    }
}

#[test]
fn test_create_command_args_struct() {
    // Test the CreateCommandArgs structure used internally
    let args = CreateCommandArgs {
        title: Some("Test Title".to_string()),
        description: Some("Test Description".to_string()),
        team: Some("ENG".to_string()),
        assignee: Some("me".to_string()),
        priority: Some(2),
        estimate: None,
        labels: vec![],
        cycle: None,
        project: None,
        project_id: None,
        from_file: None,
        open: true,
        dry_run: false,
    };

    assert_eq!(args.title, Some("Test Title".to_string()));
    assert_eq!(args.description, Some("Test Description".to_string()));
    assert_eq!(args.team, Some("ENG".to_string()));
    assert_eq!(args.assignee, Some("me".to_string()));
    assert_eq!(args.priority, Some(2));
    assert_eq!(args.from_file, None);
    assert!(args.open);
    assert!(!args.dry_run);
}

#[test]
fn test_parse_create_command_whitespace_handling() {
    use clap::Parser;

    // Test that whitespace in arguments is properly handled
    let cli = Cli::try_parse_from([
        "linear",
        "create",
        "--title",
        "  Title with spaces  ",
        "--description",
        "  Description with\nmultiple lines  ",
        "--team",
        " ENG ",
        "--assignee",
        " test@example.com ",
    ])
    .unwrap();

    match cli.command {
        Commands::Create {
            title,
            description,
            team,
            assignee,
            ..
        } => {
            // Arguments should preserve whitespace as-is (trimming is handled later)
            assert_eq!(title, Some("  Title with spaces  ".to_string()));
            assert_eq!(
                description,
                Some("  Description with\nmultiple lines  ".to_string())
            );
            assert_eq!(team, Some(" ENG ".to_string()));
            assert_eq!(assignee, Some(" test@example.com ".to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_from_file() {
    use clap::Parser;

    // Test create command with --from-file flag
    let cli = Cli::try_parse_from(["linear", "create", "--from-file", "issue.md"]).unwrap();

    match cli.command {
        Commands::Create { from_file, .. } => {
            assert_eq!(from_file, Some("issue.md".to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_from_file_short() {
    use clap::Parser;

    // Test create command with -f short flag
    let cli = Cli::try_parse_from(["linear", "create", "-f", "/path/to/issue.md"]).unwrap();

    match cli.command {
        Commands::Create { from_file, .. } => {
            assert_eq!(from_file, Some("/path/to/issue.md".to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_from_file_with_other_args() {
    use clap::Parser;

    // Test create command with --from-file and other arguments (CLI args should override)
    let cli = Cli::try_parse_from([
        "linear",
        "create",
        "--from-file",
        "issue.md",
        "--title",
        "Override Title",
        "--team",
        "OVERRIDE",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        Commands::Create {
            from_file,
            title,
            team,
            dry_run,
            ..
        } => {
            assert_eq!(from_file, Some("issue.md".to_string()));
            assert_eq!(title, Some("Override Title".to_string()));
            assert_eq!(team, Some("OVERRIDE".to_string()));
            assert!(dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_parse_create_command_empty_values() {
    use clap::Parser;

    // Test parsing with empty string values
    let cli = Cli::try_parse_from([
        "linear",
        "create",
        "--title",
        "",
        "--description",
        "",
        "--team",
        "",
        "--assignee",
        "",
    ])
    .unwrap();

    match cli.command {
        Commands::Create {
            title,
            description,
            team,
            assignee,
            ..
        } => {
            // Empty strings should still be parsed as Some("")
            assert_eq!(title, Some("".to_string()));
            assert_eq!(description, Some("".to_string()));
            assert_eq!(team, Some("".to_string()));
            assert_eq!(assignee, Some("".to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}
