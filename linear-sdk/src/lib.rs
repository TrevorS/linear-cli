// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
// Custom scalar types used by Linear's GraphQL schema
type DateTimeOrDuration = String;
type TimelessDateOrDuration = String;
type Duration = String;

#[cfg(test)]
pub mod test_helpers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issues.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListIssues;

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

pub struct LinearClient {
    client: reqwest::Client,
    base_url: String,
    _api_key: String,
}

pub struct IssueFilters {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub team: Option<String>,
}

impl LinearClient {
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

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);
        headers.insert(USER_AGENT, HeaderValue::from_static("linear-cli/0.1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url,
            _api_key: api_key,
        })
    }

    pub async fn execute_viewer_query(&self) -> Result<viewer::ResponseData> {
        let request_body = Viewer::build_query(viewer::Variables {});

        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<viewer::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        response_body
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))
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

        // Remove debug print
        // eprintln!("Full request body: {}", serde_json::to_string_pretty(&request_body).unwrap());

        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<list_issues::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        let data = response_body
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

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
        assert!(error.to_string().contains("GraphQL errors"));
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
        assert!(error.to_string().contains("GraphQL errors"));
    }

    #[tokio::test]
    async fn test_network_timeout() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .with_status(408)
            .with_body("Request Timeout")
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
        assert!(error.to_string().contains("GraphQL errors"));
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
}
