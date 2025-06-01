// ABOUTME: Terminal capability detection for image protocol support
// ABOUTME: Determines which image protocols are available in current terminal

use std::env;

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub supports_kitty_images: bool,
    pub supports_iterm2_images: bool,
    pub supports_sixel: bool,
    pub terminal_name: String,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        // Check for user override first
        if let Ok(forced_protocol) = env::var("LINEAR_CLI_FORCE_PROTOCOL") {
            return Self::from_forced_protocol(&forced_protocol);
        }

        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let term = env::var("TERM").unwrap_or_default();
        let wezterm_exe = env::var("WEZTERM_EXECUTABLE").ok();
        let kitty_window_id = env::var("KITTY_WINDOW_ID").ok();

        // Kitty protocol detection
        let supports_kitty =
            detect_kitty_support(&term_program, &term, &wezterm_exe, &kitty_window_id);

        // iTerm2 protocol detection
        let supports_iterm2 = detect_iterm2_support(&term_program, &term);

        // Sixel detection (future)
        let supports_sixel = false; // Not implementing initially

        let terminal_name = determine_terminal_name(&term_program, &term);

        Self {
            supports_kitty_images: supports_kitty,
            supports_iterm2_images: supports_iterm2,
            supports_sixel,
            terminal_name,
        }
    }

    /// Create capabilities from forced protocol override
    fn from_forced_protocol(protocol: &str) -> Self {
        let terminal_name = format!("forced-{}", protocol);

        match protocol.to_lowercase().as_str() {
            "kitty" => Self {
                supports_kitty_images: true,
                supports_iterm2_images: false,
                supports_sixel: false,
                terminal_name,
            },
            "iterm2" => Self {
                supports_kitty_images: false,
                supports_iterm2_images: true,
                supports_sixel: false,
                terminal_name,
            },
            "sixel" => Self {
                supports_kitty_images: false,
                supports_iterm2_images: false,
                supports_sixel: true,
                terminal_name,
            },
            "none" | "disable" | "disabled" => Self {
                supports_kitty_images: false,
                supports_iterm2_images: false,
                supports_sixel: false,
                terminal_name,
            },
            _ => {
                eprintln!("Warning: Unknown protocol '{}' in LINEAR_CLI_FORCE_PROTOCOL. Valid values: kitty, iterm2, sixel, none", protocol);
                Self::detect_without_override()
            }
        }
    }

    /// Detect capabilities without checking override (used for fallback)
    fn detect_without_override() -> Self {
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let term = env::var("TERM").unwrap_or_default();
        let wezterm_exe = env::var("WEZTERM_EXECUTABLE").ok();
        let kitty_window_id = env::var("KITTY_WINDOW_ID").ok();

        let supports_kitty =
            detect_kitty_support(&term_program, &term, &wezterm_exe, &kitty_window_id);
        let supports_iterm2 = detect_iterm2_support(&term_program, &term);
        let supports_sixel = false;
        let terminal_name = determine_terminal_name(&term_program, &term);

        Self {
            supports_kitty_images: supports_kitty,
            supports_iterm2_images: supports_iterm2,
            supports_sixel,
            terminal_name,
        }
    }

    pub fn supports_inline_images(&self) -> bool {
        self.supports_kitty_images || self.supports_iterm2_images || self.supports_sixel
    }

    pub fn preferred_protocol(&self) -> Option<&'static str> {
        // Prefer Kitty for terminals that support it well
        if self.supports_kitty_images {
            Some("kitty")
        } else if self.supports_iterm2_images {
            Some("iterm2")
        } else {
            None
        }
    }
}

fn detect_kitty_support(
    term_program: &str,
    term: &str,
    wezterm_exe: &Option<String>,
    kitty_window_id: &Option<String>,
) -> bool {
    // Direct Kitty terminal
    if term_program == "kitty" || kitty_window_id.is_some() {
        return true;
    }

    // WezTerm has good Kitty protocol support
    if term_program == "WezTerm" || wezterm_exe.is_some() {
        return true;
    }

    // Ghostty supports Kitty graphics protocol
    if term_program == "ghostty" {
        return true;
    }

    // Check TERM variable patterns
    if term.contains("kitty") || term.contains("ghostty") {
        return true;
    }

    // Konsole has partial support (be conservative)
    // Note: Could add version detection here

    false
}

fn detect_iterm2_support(term_program: &str, term: &str) -> bool {
    // iTerm2 itself
    if term_program == "iTerm.app" {
        return true;
    }

    // Terminals that support iTerm2 protocol
    if matches!(term_program,
        "WezTerm" |     // WezTerm supports both Kitty and iTerm2
        "mintty" |      // Windows terminal
        "Hyper" |       // Electron-based terminal
        "Warp" |        // Modern terminal with iTerm2 support
        "Tabby" |       // Cross-platform terminal
        "Terminus"      // Another modern terminal
    ) {
        return true;
    }

    // Check TERM variable for iTerm2 patterns
    if term.contains("iterm") || term.contains("iterm2") {
        return true;
    }

    // Some terminals set TERM to xterm-256color but support iTerm2
    // We could add more heuristics here based on other env vars
    false
}

fn determine_terminal_name(term_program: &str, term: &str) -> String {
    if !term_program.is_empty() {
        term_program.to_string()
    } else if !term.is_empty() {
        term.to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_kitty_terminal_detection() {
        // Save original env
        let original_term_program = env::var("TERM_PROGRAM").ok();
        let original_kitty_window = env::var("KITTY_WINDOW_ID").ok();

        // Test Kitty detection
        unsafe {
            env::set_var("TERM_PROGRAM", "kitty");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_kitty_images);
        assert_eq!(caps.preferred_protocol(), Some("kitty"));

        // Test via KITTY_WINDOW_ID
        unsafe {
            env::remove_var("TERM_PROGRAM");
            env::set_var("KITTY_WINDOW_ID", "1");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_kitty_images);

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_kitty_window {
                env::set_var("KITTY_WINDOW_ID", val);
            } else {
                env::remove_var("KITTY_WINDOW_ID");
            }
        }
    }

    #[test]
    #[serial]
    fn test_wezterm_detection() {
        let original_term_program = env::var("TERM_PROGRAM").ok();

        unsafe {
            env::set_var("TERM_PROGRAM", "WezTerm");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_kitty_images); // WezTerm supports Kitty protocol
        assert!(caps.supports_iterm2_images); // And iTerm2 protocol
        assert_eq!(caps.preferred_protocol(), Some("kitty")); // Prefers Kitty

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    #[serial]
    fn test_ghostty_detection() {
        let original_term_program = env::var("TERM_PROGRAM").ok();

        unsafe {
            env::set_var("TERM_PROGRAM", "ghostty");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_kitty_images); // Ghostty supports Kitty protocol
        assert_eq!(caps.preferred_protocol(), Some("kitty")); // Prefers Kitty

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    #[serial]
    fn test_iterm2_detection() {
        let original_term_program = env::var("TERM_PROGRAM").ok();
        let original_term = env::var("TERM").ok();
        let original_kitty_window = env::var("KITTY_WINDOW_ID").ok();
        let original_wezterm = env::var("WEZTERM_EXECUTABLE").ok();

        unsafe {
            // Clear any variables that might interfere
            env::remove_var("KITTY_WINDOW_ID");
            env::remove_var("WEZTERM_EXECUTABLE");
            env::set_var("TERM", "xterm-256color"); // Non-kitty TERM
            env::set_var("TERM_PROGRAM", "iTerm.app");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_iterm2_images);
        assert!(!caps.supports_kitty_images); // iTerm2 doesn't support Kitty
        assert_eq!(caps.preferred_protocol(), Some("iterm2"));

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_term {
                env::set_var("TERM", val);
            } else {
                env::remove_var("TERM");
            }
            if let Some(val) = original_kitty_window {
                env::set_var("KITTY_WINDOW_ID", val);
            } else {
                env::remove_var("KITTY_WINDOW_ID");
            }
            if let Some(val) = original_wezterm {
                env::set_var("WEZTERM_EXECUTABLE", val);
            } else {
                env::remove_var("WEZTERM_EXECUTABLE");
            }
        }
    }

    #[test]
    #[serial]
    fn test_no_support_detection() {
        let original_term_program = env::var("TERM_PROGRAM").ok();
        let original_term = env::var("TERM").ok();
        let original_force = env::var("LINEAR_CLI_FORCE_PROTOCOL").ok();

        unsafe {
            env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            env::set_var("TERM_PROGRAM", "unsupported");
            env::set_var("TERM", "dumb");
        }
        let caps = TerminalCapabilities::detect();
        assert!(!caps.supports_inline_images());
        assert_eq!(caps.preferred_protocol(), None);

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_term {
                env::set_var("TERM", val);
            } else {
                env::remove_var("TERM");
            }
            if let Some(val) = original_force {
                env::set_var("LINEAR_CLI_FORCE_PROTOCOL", val);
            } else {
                env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            }
        }
    }

    #[test]
    #[serial]
    fn test_force_protocol_kitty() {
        let original_force = env::var("LINEAR_CLI_FORCE_PROTOCOL").ok();
        let original_term_program = env::var("TERM_PROGRAM").ok();

        unsafe {
            env::set_var("LINEAR_CLI_FORCE_PROTOCOL", "kitty");
            env::set_var("TERM_PROGRAM", "unsupported"); // This should be ignored
        }

        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_kitty_images);
        assert!(!caps.supports_iterm2_images);
        assert_eq!(caps.preferred_protocol(), Some("kitty"));
        assert_eq!(caps.terminal_name, "forced-kitty");

        // Restore env
        unsafe {
            if let Some(val) = original_force {
                env::set_var("LINEAR_CLI_FORCE_PROTOCOL", val);
            } else {
                env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            }
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    #[serial]
    fn test_force_protocol_iterm2() {
        let original_force = env::var("LINEAR_CLI_FORCE_PROTOCOL").ok();

        unsafe {
            env::set_var("LINEAR_CLI_FORCE_PROTOCOL", "iterm2");
        }

        let caps = TerminalCapabilities::detect();
        assert!(!caps.supports_kitty_images);
        assert!(caps.supports_iterm2_images);
        assert_eq!(caps.preferred_protocol(), Some("iterm2"));
        assert_eq!(caps.terminal_name, "forced-iterm2");

        // Restore env
        unsafe {
            if let Some(val) = original_force {
                env::set_var("LINEAR_CLI_FORCE_PROTOCOL", val);
            } else {
                env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            }
        }
    }

    #[test]
    #[serial]
    fn test_force_protocol_none() {
        let original_force = env::var("LINEAR_CLI_FORCE_PROTOCOL").ok();

        unsafe {
            env::set_var("LINEAR_CLI_FORCE_PROTOCOL", "none");
        }

        let caps = TerminalCapabilities::detect();
        assert!(!caps.supports_inline_images());
        assert_eq!(caps.preferred_protocol(), None);
        assert_eq!(caps.terminal_name, "forced-none");

        // Restore env
        unsafe {
            if let Some(val) = original_force {
                env::set_var("LINEAR_CLI_FORCE_PROTOCOL", val);
            } else {
                env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            }
        }
    }

    #[test]
    #[serial]
    fn test_enhanced_iterm2_detection() {
        let original_term_program = env::var("TERM_PROGRAM").ok();
        let original_term = env::var("TERM").ok();
        let original_force = env::var("LINEAR_CLI_FORCE_PROTOCOL").ok();

        // Test Warp terminal
        unsafe {
            env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            env::set_var("TERM_PROGRAM", "Warp");
            env::set_var("TERM", "xterm-256color");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_iterm2_images);

        // Test terminal with iTerm in TERM variable
        unsafe {
            env::set_var("TERM_PROGRAM", "");
            env::set_var("TERM", "xterm-iterm2");
        }
        let caps = TerminalCapabilities::detect();
        assert!(caps.supports_iterm2_images);

        // Restore env
        unsafe {
            if let Some(val) = original_term_program {
                env::set_var("TERM_PROGRAM", val);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_term {
                env::set_var("TERM", val);
            } else {
                env::remove_var("TERM");
            }
            if let Some(val) = original_force {
                env::set_var("LINEAR_CLI_FORCE_PROTOCOL", val);
            } else {
                env::remove_var("LINEAR_CLI_FORCE_PROTOCOL");
            }
        }
    }
}
