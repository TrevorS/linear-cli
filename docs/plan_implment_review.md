# GitHub Issue Templates for Linear CLI

Copy and paste these templates directly into GitHub issues.

---

## Issue: Implement Enhanced Error Handling Architecture

### Title
Implement Enhanced Error Handling Architecture

### Body
**Priority:** P0 - Critical
**Milestone:** v2.0.0

## Description
Enhance the error handling system with better context, recovery strategies, and structured error types.

## Current State
- Basic `thiserror` in SDK and `anyhow` in CLI
- Limited error context
- No recovery strategies

## Acceptance Criteria
- [ ] Implement enhanced `LinearError` enum with detailed variants
- [ ] Add `ErrorContext` system with derive_more
- [ ] Create `ErrorContextExt` trait for adding context
- [ ] Implement `ErrorRecovery` with exponential backoff
- [ ] Add structured GraphQL error types
- [ ] Create error display helpers with suggestions

## Implementation Details

### New Dependencies
```toml
[dependencies]
derive_more = "0.99"
http = "0.2"
eyre = { version = "0.6", features = ["color-eyre"] }
backoff = "0.4"
```

### Code Structure
```rust
// linear-sdk/src/error.rs
pub enum LinearError {
    Auth { reason: Cow<'static, str>, source: Option<Box<dyn Error>> },
    IssueNotFound { identifier: String, suggestion: Option<String> },
    Network { message: String, retryable: bool, source: Box<dyn Error> },
    GraphQL { message: String, errors: Vec<GraphQLError> },
    RateLimit { reset_seconds: u64 },
}

// linear-sdk/src/error/recovery.rs
pub struct ErrorRecovery {
    backoff: ExponentialBackoff,
}
```

### Files to Create/Modify
- `linear-sdk/src/error.rs` (refactor existing)
- `linear-sdk/src/error/recovery.rs` (new)
- `linear-sdk/src/error/context.rs` (new)

---

## Issue: Extract and Organize Constants

### Title
Extract and Organize Constants

### Body
**Priority:** P0 - Critical
**Milestone:** v2.0.0

## Description
Extract all hardcoded values, magic strings, and repeated constants into a well-organized constants module.

## Current State
- Hardcoded strings scattered throughout codebase
- Magic numbers in various files
- Repeated URL patterns

## Acceptance Criteria
- [ ] Create comprehensive `constants.rs` module
- [ ] Extract all limits, timeouts, and URLs
- [ ] Add status name aliases mapping
- [ ] Define terminal sequences constants
- [ ] Create static error messages
- [ ] Use compile-time string formatting where applicable

## Implementation Details

### New Dependencies
```toml
[dependencies]
once_cell = "1.19"
const_format = "0.2"
phf = { version = "0.11", features = ["macros"] }
```

### Module Structure
```rust
// linear-cli/src/constants.rs
pub mod limits {
    pub const DEFAULT_ISSUE_LIMIT: i32 = 20;
    pub const MAX_ISSUE_LIMIT: i32 = 100;
}

pub mod timeouts {
    use std::time::Duration;
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
}

pub mod urls {
    pub const LINEAR_API_BASE: &str = "https://api.linear.app";
}

// Status aliases using once_cell
pub static STATUS_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    // Initialize mappings
});
```

### Refactoring Checklist
- [ ] Search for all hardcoded numbers
- [ ] Find all URL strings
- [ ] Identify repeated string literals
- [ ] Look for magic timeouts/durations
- [ ] Extract terminal escape sequences

---

## Issue: Implement Builder Pattern for LinearClient

### Title
Implement Builder Pattern for LinearClient

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0
**Depends on:** #1

## Description
Implement a comprehensive builder pattern for LinearClient configuration with type safety and IDE support.

## Acceptance Criteria
- [ ] Create `LinearClientConfig` struct with typed-builder
- [ ] Add TLS configuration options
- [ ] Support proxy configuration
- [ ] Implement retry configuration
- [ ] Add alternative manual builder with type states
- [ ] Use `secrecy` crate for sensitive data

## Implementation Details

### New Dependencies
```toml
[dependencies]
typed-builder = "0.18"
secrecy = "0.8"
url = "2.5"
```

### API Design
```rust
let client = LinearClient::builder()
    .auth_token(api_key)
    .verbose(true)
    .timeout(Duration::from_secs(60))
    .proxy(Some(reqwest::Proxy::https("http://proxy:8080")?))
    .build()?;
```

### Type State Example
```rust
// Compile-time enforcement of required fields
pub struct LinearClientBuilder<State = Initial> {
    config: LinearClientConfig,
    _state: PhantomData<State>,
}

// Can only build after auth is set
impl LinearClientBuilder<WithAuth> {
    pub fn build(self) -> Result<LinearClient, LinearError> { }
}
```

---

## Issue: Create GraphQL Abstraction Layer

### Title
Create GraphQL Abstraction Layer

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0
**Depends on:** #1, #2

## Description
Build a comprehensive GraphQL abstraction layer with executor trait, query builders, and caching.

## Acceptance Criteria
- [ ] Create `GraphQLExecutor` trait with tracing
- [ ] Implement `QueryBuilder` for complex queries
- [ ] Add batched query support
- [ ] Implement query caching with LRU cache
- [ ] Add proper instrumentation and logging
- [ ] Support query extensions

## Implementation Details

### New Dependencies
```toml
[dependencies]
async-trait = "0.1"
tracing = "0.1"
lru = "0.12"
parking_lot = "0.12"
ahash = "0.8"
```

### Core Trait
```rust
#[async_trait]
pub trait GraphQLExecutor: Send + Sync {
    #[instrument(skip(self, variables))]
    async fn execute<Q, V>(&self, variables: V) -> Result<Q::ResponseData, LinearError>
    where
        Q: GraphQLQuery,
        Q::ResponseData: Debug,
        V: Into<Q::Variables> + Send + Debug;
}
```

### Cache Implementation
```rust
pub struct QueryCache {
    cache: RwLock<LruCache<u64, CachedEntry>>,
    ttl: Duration,
}
```

---

## Issue: Implement Async Trait Patterns and Streaming

### Title
Implement Async Trait Patterns and Streaming

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0
**Depends on:** #4

## Description
Design comprehensive async API with streaming support for large datasets and concurrent operations.

## Acceptance Criteria
- [ ] Create `LinearApi` trait with async methods
- [ ] Implement streaming for paginated data
- [ ] Add concurrent operations support
- [ ] Implement circuit breaker pattern
- [ ] Add retry logic with backoff
- [ ] Support batch operations

## Implementation Details

### New Dependencies
```toml
[dependencies]
futures = "0.3"
tokio-stream = "0.1"
async-stream = "0.3"
pin-project = "1.1"
tower = { version = "0.4", optional = true }
```

### Streaming API
```rust
// Return a stream for large datasets
async fn stream_issues(
    &self,
    params: ListIssuesParams,
) -> Result<Pin<Box<dyn Stream<Item = Result<Issue, LinearError>> + Send>>, LinearError>;
```

### Circuit Breaker
```rust
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    config: CircuitBreakerConfig,
}
```

---

## Issue: Refactor Output Module with Terminal Detection

### Title
Refactor Output Module with Terminal Detection

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0
**Depends on:** #2

## Description
Create a comprehensive output module with terminal capability detection, theme support, and proper formatting.

## Acceptance Criteria
- [ ] Implement terminal capabilities detection
- [ ] Add color support detection (8/256/TrueColor)
- [ ] Detect hyperlink support
- [ ] Create theme system
- [ ] Add markdown rendering support
- [ ] Implement proper Unicode handling

## Implementation Details

### New Dependencies
```toml
[dependencies]
comfy-table = "7.1"
unicode-width = "0.1"
unicode-segmentation = "1.10"
supports-color = "2.1"
terminal_size = "0.3"
console = "0.15"
```

### Terminal Detection
```rust
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub hyperlink_support: bool,
    pub unicode_support: UnicodeSupport,
    pub width: Option<u16>,
    pub is_tty: bool,
}
```

### Theme System
```rust
pub static THEMES: Lazy<HashMap<&'static str, Theme>> = Lazy::new(|| {
    // Initialize built-in themes
});
```

---

## Issue: Upgrade Table Formatting

### Title
Upgrade Table Formatting

### Body
**Priority:** P2 - Medium
**Milestone:** v2.0.0
**Depends on:** #6

## Description
Replace current table implementation with comfy-table for better Unicode support and dynamic sizing.

## Acceptance Criteria
- [ ] Migrate from tabled to comfy-table
- [ ] Implement proper Unicode truncation
- [ ] Add dynamic width adjustment
- [ ] Support different table styles
- [ ] Add cell formatting based on data type
- [ ] Implement proper text wrapping

## Implementation Details

### Migration Steps
1. Replace `use tabled::*` with `use comfy_table::*`
2. Update table creation code
3. Implement Unicode-aware truncation
4. Add terminal width detection
5. Create table style presets

### Unicode Truncation
```rust
fn truncate_with_unicode(&self, s: &str, max_width: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    // Proper grapheme cluster handling
}
```

---

## Issue: Implement Type State Pattern for OAuth

### Title
Implement Type State Pattern for OAuth

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0
**Depends on:** #1

## Description
Redesign OAuth implementation using type state pattern for compile-time safety.

## Acceptance Criteria
- [ ] Create type states: Unauthenticated, Authenticating, Authenticated
- [ ] Implement state transitions at compile time
- [ ] Add secure token storage abstraction
- [ ] Implement platform-specific token storage
- [ ] Add token refresh logic
- [ ] Use `secrecy` and `zeroize` for sensitive data

## Implementation Details

### New Dependencies
```toml
[dependencies]
oauth2 = "4.4"
secrecy = "0.8"
zeroize = "1.7"
keyring = "2.3"

[target.'cfg(target_os = "macos")'.dependencies]
security-framework = "2.9"

[target.'cfg(target_os = "linux")'.dependencies]
secret-service = "3.0"

[target.'cfg(windows)'.dependencies]
windows-sys = "0.52"
```

### Type State Design
```rust
pub struct OAuthManager<State = Unauthenticated> {
    client: oauth2::Client,
    config: OAuthConfig,
    _state: PhantomData<State>,
}

// Compile-time state transitions
impl OAuthManager<Unauthenticated> {
    pub fn begin_auth_flow(self) -> (OAuthManager<Authenticating>, AuthUrl) { }
}

impl OAuthManager<Authenticating> {
    pub async fn complete_auth_flow(self, code: String) -> Result<OAuthManager<Authenticated>> { }
}
```

---

## Issue: Enhance Test Infrastructure

### Title
Enhance Test Infrastructure

### Body
**Priority:** P1 - High
**Milestone:** v2.0.0

## Description
Build comprehensive test infrastructure with builders, property testing, and better mocking.

## Acceptance Criteria
- [ ] Create test data builders with derive_builder
- [ ] Implement fixture management system
- [ ] Add property-based testing with proptest
- [ ] Create comprehensive mock service layer
- [ ] Add test helpers for common scenarios
- [ ] Implement snapshot testing for outputs

## Implementation Details

### New Dependencies
```toml
[dev-dependencies]
derive_builder = "0.12"
fake = "2.9"
proptest = "1.4"
mockall = "0.12"
rstest = "0.18"
test-case = "3.3"
criterion = "0.5"
```

### Test Builder Pattern
```rust
#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct IssueBuilder {
    #[builder(default = "self.generate_id()")]
    pub id: String,

    #[builder(default = "Faker.fake()")]
    pub title: String,
}
```

### Property Testing
```rust
proptest! {
    #[test]
    fn test_issue_serialization_roundtrip(issue in arb_issue()) {
        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(issue.id, deserialized.id);
    }
}
```

---

## Issue: Implement Configuration Management System

### Title
Implement Configuration Management System

### Body
**Priority:** P2 - Medium
**Milestone:** v2.0.0
**Depends on:** #2

## Description
Create layered configuration system with hot reloading and validation.

## Acceptance Criteria
- [ ] Implement layered config loading (defaults, system, user, local, env)
- [ ] Add config file validation
- [ ] Support TOML, YAML, JSON formats
- [ ] Implement config file watching with hot reload
- [ ] Add UI and API configuration sections
- [ ] Create feature flags system

## Implementation Details

### New Dependencies
```toml
[dependencies]
config = "0.13"
directories = "5.0"
notify = "6.1"
figment = { version = "0.10", optional = true }
```

### Configuration Layers
1. Default values (in code)
2. System config (`/etc/linear-cli/config.toml`)
3. User config (`~/.config/linear-cli/config.toml`)
4. Local config (`.linear-cli.toml`)
5. Environment variables (`LINEAR_*`)

### Hot Reload
```rust
pub struct ConfigWatcher {
    config: Arc<RwLock<AppConfig>>,
    _watcher: notify::RecommendedWatcher,
}
```

---

## Issue: Implement Command Pattern Architecture

### Title
Implement Command Pattern Architecture

### Body
**Priority:** P2 - Medium
**Milestone:** v2.0.0
**Depends on:** #1, #10

## Description
Refactor commands to use command pattern with middleware support.

## Acceptance Criteria
- [ ] Create `Command` trait with context
- [ ] Implement command registry and factory
- [ ] Add middleware pipeline support
- [ ] Create interactive command builders
- [ ] Add command validation
- [ ] Implement caching at command level

## Implementation Details

### New Dependencies
```toml
[dependencies]
dialoguer = "0.11"
console = "0.15"
edit = "0.1"
skim = "0.10"
```

### Command Pattern
```rust
#[async_trait]
pub trait Command: Debug + Send + Sync {
    async fn execute(&self, context: &mut CommandContext) -> Result<CommandOutput>;
    fn validate(&self) -> Result<()> { Ok(()) }
    fn requires_auth(&self) -> bool { true }
}
```

### Middleware System
```rust
#[async_trait]
trait Middleware: Send + Sync {
    async fn process(
        &self,
        command: &dyn Command,
        context: &mut CommandContext,
        next: Next<'_>,
    ) -> Result<CommandOutput>;
}
```

---

## Issue: Add Performance Optimizations

### Title
Add Performance Optimizations

### Body
**Priority:** P3 - Low
**Milestone:** v2.1.0
**Depends on:** All core issues

## Description
Implement various performance optimizations including string interning, zero-copy parsing, and small vector optimizations.

## Acceptance Criteria
- [ ] Implement string interning for repeated strings
- [ ] Add zero-copy deserialization where possible
- [ ] Use SmallVec for small collections
- [ ] Profile and optimize hot paths
- [ ] Consider alternative allocators
- [ ] Add compile-time optimizations

## Implementation Details

### New Dependencies
```toml
[dependencies]
string_cache = "0.8"
smallvec = "1.13"
bytes = "1.5"
indexmap = "2.1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

### Optimizations
```rust
// String interning
pub type InternedString = DefaultAtom;

// Small vector optimization
pub type LabelList = SmallVec<[Label; 4]>;

// Zero-copy deserialization
#[derive(Deserialize)]
pub struct IssueRef<'a> {
    #[serde(borrow)]
    pub id: Cow<'a, str>,
}
```

---

## Issue: Add Telemetry and Monitoring

### Title
Add Telemetry and Monitoring

### Body
**Priority:** P3 - Low
**Milestone:** v2.1.0

## Description
Add comprehensive telemetry with tracing, metrics, and optional OpenTelemetry support.

## Acceptance Criteria
- [ ] Set up tracing infrastructure
- [ ] Add metrics collection
- [ ] Implement OpenTelemetry integration (optional)
- [ ] Add performance counters
- [ ] Create debug command for diagnostics
- [ ] Add opt-in anonymous usage statistics

## Implementation Details

### New Dependencies
```toml
[dependencies]
tracing-subscriber = "0.3"
metrics = "0.21"
opentelemetry = { version = "0.21", optional = true }
```

### Tracing Setup
```rust
pub fn init_telemetry() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}
```

---

## Issue: Improve Security and Cross-Platform Support

### Title
Improve Security and Cross-Platform Support

### Body
**Priority:** P3 - Low
**Milestone:** v2.1.0

## Description
Enhance security measures and improve cross-platform compatibility.

## Acceptance Criteria
- [ ] Add constant-time string comparison for tokens
- [ ] Implement secure memory clearing
- [ ] Add platform-specific code paths
- [ ] Test on Windows and Linux
- [ ] Add security audit in CI
- [ ] Implement certificate pinning option

## Implementation Details

### New Dependencies
```toml
[dependencies]
subtle = "2.5"
human-panic = "1.2"
which = "6.0"
tempfile = "3.8"
fslock = "0.2"
```

### Security Enhancements
```rust
// Constant-time comparison
use subtle::ConstantTimeEq;

pub fn verify_token(provided: &[u8], expected: &[u8]) -> bool {
    provided.ct_eq(expected).into()
}
```

### Platform-Specific Code
```rust
#[cfg(target_os = "windows")]
mod windows {
    pub fn enable_ansi_support() { }
}

#[cfg(unix)]
mod unix {
    pub fn get_terminal_size() -> Option<(u16, u16)> { }
}
```
