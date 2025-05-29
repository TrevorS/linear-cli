// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

pub mod error;
pub mod retry;

#[cfg(feature = "oauth")]
pub mod oauth;

#[cfg(feature = "oauth")]
pub mod storage;

pub use error::LinearError;
pub use retry::RetryConfig;

pub type Result<T> = std::result::Result<T, LinearError>;
// Custom scalar types used by Linear's GraphQL schema
type DateTimeOrDuration = String;
type TimelessDateOrDuration = String;
type Duration = String;
type DateTime = String;

#[cfg(test)]
pub mod test_helpers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug",
    skip_serializing_none
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issues.graphql",
    response_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct ListIssues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issue.graphql",
    response_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct GetIssue;

pub use viewer::ResponseData as ViewerResponseData;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub assignee_id: Option<String>,
    pub team: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: IssueState,
    pub assignee: Option<IssueAssignee>,
    pub team: Option<IssueTeam>,
    pub project: Option<IssueProject>,
    pub labels: Vec<IssueLabel>,
    pub priority: Option<i64>,
    pub priority_label: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueState {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssignee {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTeam {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueProject {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueLabel {
    pub name: String,
    pub color: String,
}

pub struct LinearClient {
    client: reqwest::Client,
    base_url: String,
    _auth_token: String,
    verbose: bool,
    retry_config: retry::RetryConfig,
}

pub struct IssueFilters {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub team: Option<String>,
}

impl LinearClient {
    fn extract_issue_id(&self, error_string: &str) -> String {
        // Try to extract issue ID from error message
        if let Some(start) = error_string.find("Issue ") {
            if let Some(end) = error_string[start + 6..].find(" ") {
                return error_string[start + 6..start + 6 + end].to_string();
            }
        }
        "unknown".to_string()
    }

    async fn build_issue_filter(
        &self,
        filters: &IssueFilters,
    ) -> Result<Option<list_issues::IssueFilter>> {
        use list_issues::*;

        // Create a minimal issue filter with only the fields we need
        let mut issue_filter = IssueFilter {
            id: None,
            created_at: None,
            updated_at: None,
            number: None,
            title: None,
            description: None,
            priority: None,
            estimate: None,
            started_at: None,
            triaged_at: None,
            completed_at: None,
            canceled_at: None,
            archived_at: None,
            auto_closed_at: None,
            auto_archived_at: None,
            added_to_cycle_at: None,
            added_to_cycle_period: None,
            due_date: None,
            snoozed_until_at: None,
            assignee: Box::new(None),
            last_applied_template: None,
            recurring_issue_template: None,
            source_metadata: None,
            creator: Box::new(None),
            parent: Box::new(None),
            snoozed_by: Box::new(None),
            labels: Box::new(None),
            subscribers: Box::new(None),
            team: Box::new(None),
            project_milestone: None,
            comments: Box::new(None),
            cycle: Box::new(None),
            project: Box::new(None),
            state: Box::new(None),
            children: Box::new(None),
            attachments: Box::new(None),
            searchable_content: None,
            has_related_relations: None,
            has_duplicate_relations: None,
            has_blocked_by_relations: None,
            has_blocking_relations: None,
            sla_status: None,
            reactions: None,
            needs: Box::new(None),
            customer_count: None,
            lead_time: None,
            cycle_time: None,
            age_time: None,
            triage_time: None,
            and: Box::new(None),
            or: Box::new(None),
        };

        let mut has_filters = false;

        // Handle assignee filter
        if let Some(assignee_value) = &filters.assignee {
            match assignee_value.as_str() {
                "me" => {
                    // We need to query for the current user's ID
                    let viewer_data = self.execute_viewer_query().await?;
                    let viewer_id = viewer_data.viewer.id;

                    // Create a minimal filter with only the fields we need
                    issue_filter.assignee = Box::new(Some(NullableUserFilter {
                        id: Some(IDComparator {
                            eq: Some(viewer_id),
                            neq: None,
                            in_: None,
                            nin: None,
                        }),
                        created_at: None,
                        updated_at: None,
                        name: None,
                        display_name: None,
                        email: None,
                        active: None,
                        assigned_issues: Box::new(None),
                        admin: None,
                        invited: None,
                        app: None,
                        is_me: None,
                        null: None,
                        and: Box::new(None),
                        or: Box::new(None),
                    }));
                    has_filters = true;
                }
                "unassigned" => {
                    // For unassigned, we only need the null field
                    issue_filter.assignee = Box::new(Some(NullableUserFilter {
                        id: None,
                        created_at: None,
                        updated_at: None,
                        name: None,
                        display_name: None,
                        email: None,
                        active: None,
                        assigned_issues: Box::new(None),
                        admin: None,
                        invited: None,
                        app: None,
                        is_me: None,
                        null: Some(true),
                        and: Box::new(None),
                        or: Box::new(None),
                    }));
                    has_filters = true;
                }
                _ => {
                    // Filter by assignee name
                    issue_filter.assignee = Box::new(Some(NullableUserFilter {
                        id: None,
                        created_at: None,
                        updated_at: None,
                        name: Some(StringComparator {
                            eq: Some(assignee_value.clone()),
                            neq: None,
                            in_: None,
                            nin: None,
                            eq_ignore_case: None,
                            neq_ignore_case: None,
                            starts_with: None,
                            starts_with_ignore_case: None,
                            not_starts_with: None,
                            ends_with: None,
                            not_ends_with: None,
                            contains: None,
                            contains_ignore_case: None,
                            not_contains: None,
                            not_contains_ignore_case: None,
                            contains_ignore_case_and_accent: None,
                        }),
                        display_name: None,
                        email: None,
                        active: None,
                        assigned_issues: Box::new(None),
                        admin: None,
                        invited: None,
                        app: None,
                        is_me: None,
                        null: None,
                        and: Box::new(None),
                        or: Box::new(None),
                    }));
                    has_filters = true;
                }
            }
        }

        // Handle status filter
        if let Some(status_value) = &filters.status {
            let normalized_status = Self::normalize_status(status_value);
            issue_filter.state = Box::new(Some(WorkflowStateFilter {
                id: None,
                created_at: None,
                updated_at: None,
                name: Some(StringComparator {
                    eq: Some(normalized_status),
                    neq: None,
                    in_: None,
                    nin: None,
                    eq_ignore_case: None,
                    neq_ignore_case: None,
                    starts_with: None,
                    starts_with_ignore_case: None,
                    not_starts_with: None,
                    ends_with: None,
                    not_ends_with: None,
                    contains: None,
                    contains_ignore_case: None,
                    not_contains: None,
                    not_contains_ignore_case: None,
                    contains_ignore_case_and_accent: None,
                }),
                description: None,
                position: None,
                type_: None,
                team: Box::new(None),
                issues: Box::new(None),
                and: Box::new(None),
                or: Box::new(None),
            }));
            has_filters = true;
        }

        // Handle team filter
        if let Some(team_value) = &filters.team {
            issue_filter.team = Box::new(Some(TeamFilter {
                id: None,
                created_at: None,
                updated_at: None,
                name: None,
                key: Some(StringComparator {
                    eq: Some(team_value.clone()),
                    neq: None,
                    in_: None,
                    nin: None,
                    eq_ignore_case: None,
                    neq_ignore_case: None,
                    starts_with: None,
                    starts_with_ignore_case: None,
                    not_starts_with: None,
                    ends_with: None,
                    not_ends_with: None,
                    contains: None,
                    contains_ignore_case: None,
                    not_contains: None,
                    not_contains_ignore_case: None,
                    contains_ignore_case_and_accent: None,
                }),
                description: None,
                issues: Box::new(None),
                parent: Box::new(None),
                and: Box::new(None),
                or: Box::new(None),
            }));
            has_filters = true;
        }

        if has_filters {
            Ok(Some(issue_filter))
        } else {
            Ok(None)
        }
    }

    fn normalize_status(status: &str) -> String {
        match status.to_lowercase().as_str() {
            "todo" => "Todo".to_string(),
            "in progress" | "inprogress" | "in_progress" => "In Progress".to_string(),
            "done" => "Done".to_string(),
            _ => {
                // Return the title case version for unknown statuses
                let mut result = String::new();
                let mut capitalize_next = true;
                for c in status.chars() {
                    if c.is_whitespace() {
                        result.push(c);
                        capitalize_next = true;
                    } else if capitalize_next {
                        result.push(c.to_uppercase().next().unwrap_or(c));
                        capitalize_next = false;
                    } else {
                        result.push(c.to_lowercase().next().unwrap_or(c));
                    }
                }
                result
            }
        }
    }
    pub fn new(api_key: String) -> Result<Self> {
        Self::with_base_url(api_key, "https://api.linear.app".to_string())
    }

    pub fn new_with_verbose(api_key: String, verbose: bool) -> Result<Self> {
        Self::with_base_url_and_verbose(api_key, "https://api.linear.app".to_string(), verbose)
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self> {
        Self::with_base_url_and_verbose(api_key, base_url, false)
    }

    pub fn with_base_url_and_verbose(
        auth_token: String,
        base_url: String,
        verbose: bool,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_token).map_err(|_| LinearError::Auth)?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("linear-cli/0.1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(LinearError::from)?;

        Ok(Self {
            client,
            base_url,
            _auth_token: auth_token,
            verbose,
            retry_config: retry::RetryConfig::default(),
        })
    }

    #[cfg(feature = "oauth")]
    pub fn new_with_oauth_token(oauth_token: String) -> Result<Self> {
        // OAuth tokens need "Bearer " prefix
        let bearer_token = format!("Bearer {}", oauth_token);
        Self::with_base_url_and_verbose(bearer_token, "https://api.linear.app".to_string(), false)
    }

    #[cfg(feature = "oauth")]
    pub fn new_with_oauth_token_and_verbose(oauth_token: String, verbose: bool) -> Result<Self> {
        // OAuth tokens need "Bearer " prefix
        let bearer_token = format!("Bearer {}", oauth_token);
        Self::with_base_url_and_verbose(bearer_token, "https://api.linear.app".to_string(), verbose)
    }

    pub async fn execute_viewer_query(&self) -> Result<viewer::ResponseData> {
        let request_body = Viewer::build_query(viewer::Variables {});

        if self.verbose {
            eprintln!("Sending GraphQL query: viewer");
            eprintln!(
                "Request body: {}",
                serde_json::to_string_pretty(&request_body).unwrap_or_default()
            );
        }

        retry::retry_with_backoff(&self.retry_config, self.verbose, || {
            let client = &self.client;
            let base_url = &self.base_url;
            let request_body = &request_body;
            let verbose = self.verbose;

            async move {
                let start_time = std::time::Instant::now();
                let response = client
                    .post(format!("{}/graphql", base_url))
                    .json(request_body)
                    .send()
                    .await
                    .map_err(LinearError::from)?;

                if verbose {
                    eprintln!("Request completed in {:?}", start_time.elapsed());
                    eprintln!("Response status: {}", response.status());
                }

                let response_body: Response<viewer::ResponseData> =
                    response.json().await.map_err(LinearError::from)?;

                if let Some(errors) = response_body.errors {
                    return Err(LinearError::GraphQL(format!("{:?}", errors)));
                }

                response_body.data.ok_or(LinearError::InvalidResponse)
            }
        })
        .await
    }

    pub async fn list_issues(&self, limit: i32) -> Result<Vec<Issue>> {
        self.list_issues_with_filter(limit, None).await
    }

    pub async fn list_issues_filtered(
        &self,
        limit: i32,
        filters: Option<IssueFilters>,
    ) -> Result<Vec<Issue>> {
        let graphql_filter = if let Some(filters) = filters {
            self.build_issue_filter(&filters).await?
        } else {
            None
        };

        self.list_issues_with_filter(limit, graphql_filter).await
    }

    pub async fn list_issues_with_filter(
        &self,
        limit: i32,
        filter: Option<list_issues::IssueFilter>,
    ) -> Result<Vec<Issue>> {
        let request_body = ListIssues::build_query(list_issues::Variables {
            first: limit as i64,
            filter,
        });

        if self.verbose {
            eprintln!("Sending GraphQL query: listIssues (limit: {})", limit);
            eprintln!(
                "Request body: {}",
                serde_json::to_string_pretty(&request_body).unwrap_or_default()
            );
        }

        let start_time = std::time::Instant::now();
        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(LinearError::from)?;

        if self.verbose {
            eprintln!("Request completed in {:?}", start_time.elapsed());
            eprintln!("Response status: {}", response.status());
        }

        let response_body: Response<list_issues::ResponseData> =
            response.json().await.map_err(LinearError::from)?;

        if let Some(errors) = response_body.errors {
            return Err(LinearError::GraphQL(format!("{:?}", errors)));
        }

        let data = response_body.data.ok_or(LinearError::InvalidResponse)?;

        let issues = data
            .issues
            .nodes
            .into_iter()
            .map(|issue| Issue {
                id: issue.id,
                identifier: issue.identifier,
                title: issue.title,
                status: issue.state.name,
                assignee: issue.assignee.as_ref().map(|a| a.name.clone()),
                assignee_id: issue.assignee.map(|a| a.id),
                team: Some(issue.team.key),
            })
            .collect();

        Ok(issues)
    }

    pub async fn get_issue(&self, id: String) -> Result<DetailedIssue> {
        let request_body = GetIssue::build_query(get_issue::Variables { id: id.clone() });

        if self.verbose {
            eprintln!("Sending GraphQL query: getIssue (id: {})", id);
            eprintln!(
                "Request body: {}",
                serde_json::to_string_pretty(&request_body).unwrap_or_default()
            );
        }

        let start_time = std::time::Instant::now();
        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(LinearError::from)?;

        if self.verbose {
            eprintln!("Request completed in {:?}", start_time.elapsed());
            eprintln!("Response status: {}", response.status());
        }

        let response_body: Response<get_issue::ResponseData> =
            response.json().await.map_err(LinearError::from)?;

        if let Some(errors) = response_body.errors {
            // Check if this is a "not found" error
            let error_string = format!("{:?}", errors);
            if error_string.contains("not found") || error_string.contains("not exist") {
                return Err(LinearError::IssueNotFound(
                    self.extract_issue_id(&error_string),
                ));
            }
            return Err(LinearError::GraphQL(error_string));
        }

        let data = response_body.data.ok_or(LinearError::InvalidResponse)?;

        let issue = data.issue;

        Ok(DetailedIssue {
            id: issue.id,
            identifier: issue.identifier,
            title: issue.title,
            description: issue.description,
            state: IssueState {
                name: issue.state.name,
                type_: issue.state.type_,
            },
            assignee: issue.assignee.map(|a| IssueAssignee {
                name: a.name,
                email: a.email,
            }),
            team: Some(IssueTeam {
                key: issue.team.key,
                name: issue.team.name,
            }),
            project: issue.project.map(|p| IssueProject { name: p.name }),
            labels: issue
                .labels
                .nodes
                .into_iter()
                .map(|l| IssueLabel {
                    name: l.name,
                    color: l.color,
                })
                .collect(),
            priority: Some(issue.priority as i64),
            priority_label: Some(issue.priority_label),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            url: issue.url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_normalize_status() {
        // Test basic status normalization
        assert_eq!(LinearClient::normalize_status("todo"), "Todo");
        assert_eq!(LinearClient::normalize_status("TODO"), "Todo");
        assert_eq!(LinearClient::normalize_status("ToDo"), "Todo");

        assert_eq!(LinearClient::normalize_status("done"), "Done");
        assert_eq!(LinearClient::normalize_status("DONE"), "Done");
        assert_eq!(LinearClient::normalize_status("Done"), "Done");

        assert_eq!(LinearClient::normalize_status("in progress"), "In Progress");
        assert_eq!(LinearClient::normalize_status("IN PROGRESS"), "In Progress");
        assert_eq!(LinearClient::normalize_status("in_progress"), "In Progress");
        assert_eq!(LinearClient::normalize_status("inprogress"), "In Progress");

        // Test unknown status - should title case
        assert_eq!(LinearClient::normalize_status("blocked"), "Blocked");
        assert_eq!(LinearClient::normalize_status("BLOCKED"), "Blocked");
        assert_eq!(
            LinearClient::normalize_status("waiting for review"),
            "Waiting For Review"
        );
    }

    #[test]
    fn test_linear_client_creation() {
        let client = LinearClient::new("test_api_key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_viewer_query_builds() {
        let _query = Viewer::build_query(viewer::Variables {});
    }

    #[tokio::test]
    async fn test_successful_api_call() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .match_header("user-agent", "linear-cli/0.1.0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_viewer_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.viewer.id, "test-user-id");
        assert_eq!(data.viewer.name, "Test User");
        assert_eq!(data.viewer.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_authentication_error() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "invalid_key")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(mock_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("invalid_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL error"));
    }

    #[tokio::test]
    async fn test_graphql_errors() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_graphql_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL error"));
    }

    #[tokio::test]
    async fn test_network_timeout() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .with_status(408)
            .with_body("Request Timeout")
            .expect(4)  // 1 initial + 3 retries
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::new(api_key).expect("Failed to create client");
        let result = client.execute_viewer_query().await;

        assert!(result.is_ok(), "Query should succeed with valid API key");
        let data = result.unwrap();
        assert!(!data.viewer.id.is_empty(), "Viewer should have an ID");
    }

    #[test]
    fn test_list_issues_query_builds() {
        let _query = ListIssues::build_query(list_issues::Variables {
            first: 20,
            filter: None,
        });
    }

    #[tokio::test]
    async fn test_list_issues_success() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_issues_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 3);

        assert_eq!(issues[0].identifier, "TEST-1");
        assert_eq!(issues[0].title, "Test Issue 1");
        assert_eq!(issues[0].status, "Todo");
        assert_eq!(issues[0].assignee, Some("Alice".to_string()));
        assert_eq!(issues[0].assignee_id, Some("user-1".to_string()));
        assert_eq!(issues[0].team, Some("ENG".to_string()));

        assert_eq!(issues[1].identifier, "TEST-2");
        assert_eq!(issues[1].title, "Test Issue 2");
        assert_eq!(issues[1].status, "In Progress");
        assert_eq!(issues[1].assignee, Some("Bob".to_string()));
        assert_eq!(issues[1].assignee_id, Some("user-2".to_string()));
        assert_eq!(issues[1].team, Some("DESIGN".to_string()));

        assert_eq!(issues[2].identifier, "TEST-3");
        assert_eq!(issues[2].title, "Test Issue 3");
        assert_eq!(issues[2].status, "Done");
        assert_eq!(issues[2].assignee, None);
        assert_eq!(issues[2].assignee_id, None);
        assert_eq!(issues[2].team, Some("QA".to_string()));
    }

    #[tokio::test]
    async fn test_list_issues_empty() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_empty_issues_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_list_issues_error() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_graphql_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL error"));
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_list_issues_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::new(api_key).expect("Failed to create client");
        let result = client.list_issues(5).await;

        assert!(result.is_ok(), "Query should succeed with valid API key");
        let issues = result.unwrap();
        assert!(issues.len() <= 5, "Should return at most 5 issues");
    }

    #[tokio::test]
    async fn test_build_issue_filter_assignee_me() {
        let mut server = mock_linear_server().await;

        // Mock the viewer query
        let viewer_mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_viewer_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();

        let filters = IssueFilters {
            assignee: Some("me".to_string()),
            status: None,
            team: None,
        };

        let result = client.build_issue_filter(&filters).await;
        viewer_mock.assert();

        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_some());

        let filter = filter.unwrap();
        let assignee_filter = (*filter.assignee).as_ref().unwrap();
        assert!(assignee_filter.id.is_some());
        assert_eq!(
            assignee_filter.id.as_ref().unwrap().eq,
            Some("test-user-id".to_string())
        );
    }

    #[tokio::test]
    async fn test_build_issue_filter_assignee_unassigned() {
        let client = LinearClient::new("test_api_key".to_string()).unwrap();

        let filters = IssueFilters {
            assignee: Some("unassigned".to_string()),
            status: None,
            team: None,
        };

        let result = client.build_issue_filter(&filters).await;
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_some());

        let filter = filter.unwrap();
        let assignee_filter = (*filter.assignee).as_ref().unwrap();
        assert_eq!(assignee_filter.null, Some(true));
    }

    #[tokio::test]
    async fn test_build_issue_filter_status() {
        let client = LinearClient::new("test_api_key".to_string()).unwrap();

        let filters = IssueFilters {
            assignee: None,
            status: Some("in progress".to_string()),
            team: None,
        };

        let result = client.build_issue_filter(&filters).await;
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_some());

        let filter = filter.unwrap();
        let state_filter = (*filter.state).as_ref().unwrap();
        assert!(state_filter.name.is_some());
        assert_eq!(
            state_filter.name.as_ref().unwrap().eq,
            Some("In Progress".to_string())
        );
    }

    #[tokio::test]
    async fn test_build_issue_filter_team() {
        let client = LinearClient::new("test_api_key".to_string()).unwrap();

        let filters = IssueFilters {
            assignee: None,
            status: None,
            team: Some("ENG".to_string()),
        };

        let result = client.build_issue_filter(&filters).await;
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_some());

        let filter = filter.unwrap();
        let team_filter = (*filter.team).as_ref().unwrap();
        assert!(team_filter.key.is_some());
        assert_eq!(
            team_filter.key.as_ref().unwrap().eq,
            Some("ENG".to_string())
        );
    }

    #[tokio::test]
    async fn test_build_issue_filter_combined() {
        let mut server = mock_linear_server().await;

        // Mock the viewer query
        let viewer_mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_viewer_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();

        let filters = IssueFilters {
            assignee: Some("me".to_string()),
            status: Some("todo".to_string()),
            team: Some("DESIGN".to_string()),
        };

        let result = client.build_issue_filter(&filters).await;
        viewer_mock.assert();

        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_some());

        let filter = filter.unwrap();

        // Check assignee filter
        let assignee_filter = (*filter.assignee).as_ref().unwrap();
        assert_eq!(
            assignee_filter.id.as_ref().unwrap().eq,
            Some("test-user-id".to_string())
        );

        // Check status filter
        let state_filter = (*filter.state).as_ref().unwrap();
        assert_eq!(
            state_filter.name.as_ref().unwrap().eq,
            Some("Todo".to_string())
        );

        // Check team filter
        let team_filter = (*filter.team).as_ref().unwrap();
        assert_eq!(
            team_filter.key.as_ref().unwrap().eq,
            Some("DESIGN".to_string())
        );
    }

    #[tokio::test]
    async fn test_build_issue_filter_empty() {
        let client = LinearClient::new("test_api_key".to_string()).unwrap();

        let filters = IssueFilters {
            assignee: None,
            status: None,
            team: None,
        };

        let result = client.build_issue_filter(&filters).await;
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.is_none());
    }

    #[test]
    fn test_get_issue_query_builds() {
        let _query = GetIssue::build_query(get_issue::Variables {
            id: "ENG-123".to_string(),
        });
    }

    #[tokio::test]
    async fn test_get_issue_success() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_detailed_issue_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.get_issue("ENG-123".to_string()).await;

        mock.assert();
        assert!(result.is_ok());
        let issue = result.unwrap();

        assert_eq!(issue.identifier, "ENG-123");
        assert_eq!(issue.title, "Fix login race condition");
        assert_eq!(issue.state.name, "In Progress");
        assert_eq!(issue.state.type_, "started");

        assert!(issue.assignee.is_some());
        let assignee = issue.assignee.unwrap();
        assert_eq!(assignee.name, "John Doe");
        assert_eq!(assignee.email, "john@example.com");

        assert!(issue.team.is_some());
        let team = issue.team.unwrap();
        assert_eq!(team.key, "ENG");
        assert_eq!(team.name, "Engineering");

        assert!(issue.project.is_some());
        assert_eq!(issue.project.unwrap().name, "Web App");

        assert_eq!(issue.labels.len(), 2);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.labels[1].name, "authentication");

        assert_eq!(issue.priority, Some(2));
        assert_eq!(issue.priority_label, Some("High".to_string()));

        assert!(issue.description.is_some());
        assert!(
            issue
                .description
                .unwrap()
                .contains("race conditions when logging in")
        );

        assert_eq!(issue.url, "https://linear.app/test/issue/ENG-123");
    }

    #[tokio::test]
    async fn test_get_issue_minimal_response() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_minimal_issue_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.get_issue("ENG-456".to_string()).await;

        mock.assert();
        assert!(result.is_ok());
        let issue = result.unwrap();

        assert_eq!(issue.identifier, "ENG-456");
        assert_eq!(issue.title, "Simple issue");
        assert_eq!(issue.state.name, "Todo");
        assert!(issue.assignee.is_none());
        assert!(issue.project.is_none());
        assert!(issue.description.is_none());
        assert_eq!(issue.labels.len(), 0);
    }

    #[tokio::test]
    async fn test_get_issue_error() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_graphql_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.get_issue("INVALID".to_string()).await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL error"));
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_get_issue_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::new(api_key).expect("Failed to create client");

        // Try to get a specific issue - this will vary by team
        // In a real test, you'd use a known issue ID
        let result = client.get_issue("ENG-1".to_string()).await;

        // The test should either succeed or fail with "Issue not found"
        // but not with authentication or network errors
        match result {
            Ok(_) => {
                // Success case - issue exists
            }
            Err(e) => {
                // Should be a GraphQL error about the issue not existing
                assert!(
                    e.to_string().contains("GraphQL errors")
                        || e.to_string().contains("Issue not found")
                );
            }
        }
    }
}
