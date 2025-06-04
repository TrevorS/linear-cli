// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::borrow::Cow;
use std::fmt::Debug;

pub mod builder;
pub mod constants;
pub mod error;
pub mod graphql;
pub mod retry;

pub use builder::LinearClientConfig;
use constants::urls;
use secrecy::ExposeSecret;

pub use builder::{Initial, LinearClientConfigBuilder, TypedLinearClientBuilder, WithAuth};
pub use graphql::{GraphQLExecutor, QueryBuilder};

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
type TimelessDate = String;
type Duration = String;
type DateTime = String;
#[allow(clippy::upper_case_acronyms)]
type JSON = serde_json::Value;

#[cfg(test)]
pub mod test_helpers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issues.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct ListIssues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issue.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct GetIssue;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/mutations/create_issue.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct CreateIssue;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/users.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct ListUsers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/teams.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    skip_serializing_none
)]
pub struct ListTeams;

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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: IssueState,
    pub assignee: Option<IssueAssignee>,
    pub team: Option<IssueTeam>,
    pub labels: Vec<IssueLabel>,
    pub priority: Option<i64>,
    pub priority_label: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct CreateIssueInput {
    pub title: String,
    pub description: Option<String>,
    pub team_id: Option<String>,
    pub assignee_id: Option<String>,
    pub priority: Option<i64>,
    pub label_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub email: String,
    pub active: bool,
    pub guest: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<User>,
}

pub struct LinearClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    _auth_token: String,
    pub(crate) verbose: bool,
    pub(crate) retry_config: retry::RetryConfig,
    _max_retries: usize,
}

pub struct IssueFilters {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub team: Option<String>,
}

impl LinearClient {
    pub fn from_config(config: LinearClientConfig) -> Result<Self> {
        let auth_token = config.auth_token.expose_secret();

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(auth_token).map_err(|_| LinearError::Auth {
                reason: Cow::Borrowed("Invalid API key format"),
                source: None,
            })?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("linear-cli/0.1.0"));

        let mut client_builder = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(config.timeout);

        if let Some(proxy) = config.proxy {
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder.build().map_err(LinearError::from)?;

        let retry_config = retry::RetryConfig {
            max_retries: config.max_retries as u32,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(10),
            backoff_multiplier: 2.0,
        };

        Ok(Self {
            client,
            base_url: config
                .base_url
                .unwrap_or_else(|| urls::LINEAR_API_BASE.to_string()),
            _auth_token: auth_token.to_string(),
            verbose: config.verbose,
            retry_config,
            _max_retries: config.max_retries,
        })
    }

    /// Execute a GraphQL query using the abstraction layer
    async fn execute_graphql<Q, V>(&self, variables: V) -> Result<Q::ResponseData>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
        V: Into<Q::Variables> + Send + Debug,
    {
        let variables = variables.into();

        // Build the query request
        let request_body = Q::build_query(variables);

        if self.verbose {
            let query_name = std::any::type_name::<Q>()
                .split("::")
                .last()
                .unwrap_or("unknown");
            log::debug!("Sending GraphQL query: {}", query_name);
            log::debug!(
                "Request body: {}",
                serde_json::to_string_pretty(&request_body).unwrap_or_default()
            );
        }

        // Execute using the client's retry logic
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
                    log::debug!("Request completed in {:?}", start_time.elapsed());
                    log::debug!("Response status: {}", response.status());
                }

                // Check for HTTP error status codes
                if !response.status().is_success() {
                    return Err(LinearError::from_status(
                        http::StatusCode::from_u16(response.status().as_u16()).unwrap(),
                    ));
                }

                let response_body: Response<Q::ResponseData> =
                    response.json().await.map_err(LinearError::from)?;

                if let Some(errors) = response_body.errors {
                    return Err(LinearError::GraphQL {
                        message: format!("{:?}", errors),
                        errors: vec![],
                    });
                }

                response_body.data.ok_or(LinearError::InvalidResponse)
            }
        })
        .await
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

    pub async fn execute_viewer_query(&self) -> Result<viewer::ResponseData> {
        self.execute_graphql::<Viewer, _>(viewer::Variables {})
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
        let variables = list_issues::Variables {
            first: limit as i64,
            filter,
        };

        let data = self.execute_graphql::<ListIssues, _>(variables).await?;
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
        let variables = get_issue::Variables { id: id.clone() };

        // Execute query through the GraphQL abstraction layer with error handling
        let data = match self.execute_graphql::<GetIssue, _>(variables).await {
            Ok(data) => data,
            Err(LinearError::GraphQL { message, .. }) => {
                // Check if this is a "not found" error
                if message.contains("not found") || message.contains("not exist") {
                    return Err(LinearError::IssueNotFound {
                        identifier: id,
                        suggestion: None,
                    });
                }
                return Err(LinearError::GraphQL {
                    message,
                    errors: vec![],
                });
            }
            Err(e) => return Err(e),
        };

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

    pub async fn create_issue(&self, input: CreateIssueInput) -> Result<CreatedIssue> {
        // For now, let's create a simpler implementation that we'll enhance in later phases
        // This builds the actual GraphQL variables properly

        let variables = create_issue::Variables {
            input: create_issue::IssueCreateInput {
                title: Some(input.title),
                description: input.description,
                team_id: input.team_id.unwrap_or_default(),
                assignee_id: input.assignee_id,
                priority: input.priority,
                label_ids: input.label_ids,

                // Initialize other available fields to sensible defaults
                state_id: None,
                project_id: None,
                project_milestone_id: None,
                parent_id: None,
                due_date: None,
                estimate: None,
                sort_order: None,
                create_as_user: None,
                display_icon_url: None,
                subscriber_ids: None,
                template_id: None,
                completed_at: None,
                created_at: None,
                cycle_id: None,
                sla_breaches_at: None,
                sla_started_at: None,
                id: None,
                description_data: None,
                last_applied_template_id: None,
                reference_comment_id: None,
                source_comment_id: None,
                preserve_sort_order_on_create: None,
                priority_sort_order: None,
                sla_type: None,
                source_pull_request_comment_id: None,
                sub_issue_sort_order: None,
            },
        };

        let data = self.execute_graphql::<CreateIssue, _>(variables).await?;

        if !data.issue_create.success {
            return Err(LinearError::GraphQL {
                message: "Issue creation failed".to_string(),
                errors: vec![],
            });
        }

        let issue = data
            .issue_create
            .issue
            .ok_or_else(|| LinearError::GraphQL {
                message: "Issue creation succeeded but no issue data returned".to_string(),
                errors: vec![],
            })?;

        Ok(CreatedIssue {
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

    pub async fn list_users(&self, limit: i32) -> Result<Vec<User>> {
        self.list_users_filtered(limit, None).await
    }

    pub async fn list_users_filtered(
        &self,
        limit: i32,
        filter: Option<list_users::UserFilter>,
    ) -> Result<Vec<User>> {
        let variables = list_users::Variables {
            first: limit as i64,
            filter,
        };

        let data = self.execute_graphql::<ListUsers, _>(variables).await?;
        let users = data
            .users
            .nodes
            .into_iter()
            .map(|user| User {
                id: user.id,
                name: user.name,
                display_name: Some(user.display_name),
                email: user.email,
                active: user.active,
                guest: user.guest,
            })
            .collect();
        Ok(users)
    }

    pub async fn list_teams(&self) -> Result<Vec<Team>> {
        let variables = list_teams::Variables {
            first: 50, // Reasonable default for teams
        };

        let data = self.execute_graphql::<ListTeams, _>(variables).await?;
        let teams = data
            .teams
            .nodes
            .into_iter()
            .map(|team| Team {
                id: team.id,
                key: team.key,
                name: team.name,
                description: team.description,
                members: team
                    .members
                    .nodes
                    .into_iter()
                    .map(|member| User {
                        id: member.id,
                        name: member.name,
                        display_name: Some(member.display_name),
                        email: member.email,
                        active: member.active,
                        guest: false, // Team members are not guests by default
                    })
                    .collect(),
            })
            .collect();
        Ok(teams)
    }

    pub async fn search_users(&self, query: &str, limit: i32) -> Result<Vec<User>> {
        use list_users::*;

        // Create a user filter for searching by name or email
        let user_filter = UserFilter {
            and: Box::new(None),
            or: Box::new(Some(vec![
                UserFilter {
                    name: Some(StringComparator {
                        contains_ignore_case: Some(query.to_string()),
                        eq: None,
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
                        not_contains: None,
                        not_contains_ignore_case: None,
                        contains_ignore_case_and_accent: None,
                    }),
                    and: Box::new(None),
                    or: Box::new(None),
                    id: None,
                    created_at: None,
                    updated_at: None,
                    display_name: None,
                    email: None,
                    active: None,
                    admin: None,
                    invited: None,
                    assigned_issues: Box::new(None),
                    app: None,
                    is_me: None,
                },
                UserFilter {
                    email: Some(StringComparator {
                        contains_ignore_case: Some(query.to_string()),
                        eq: None,
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
                        not_contains: None,
                        not_contains_ignore_case: None,
                        contains_ignore_case_and_accent: None,
                    }),
                    and: Box::new(None),
                    or: Box::new(None),
                    id: None,
                    created_at: None,
                    updated_at: None,
                    name: None,
                    display_name: None,
                    active: None,
                    admin: None,
                    invited: None,
                    assigned_issues: Box::new(None),
                    app: None,
                    is_me: None,
                },
            ])),
            id: None,
            created_at: None,
            updated_at: None,
            name: None,
            display_name: None,
            email: None,
            active: None,
            admin: None,
            invited: None,
            assigned_issues: Box::new(None),
            app: None,
            is_me: None,
        };

        self.list_users_filtered(limit, Some(user_filter)).await
    }

    pub async fn resolve_team_key_to_id(&self, team_key: &str) -> Result<String> {
        let teams = self.list_teams().await?;

        for team in teams {
            if team.key.eq_ignore_ascii_case(team_key) {
                return Ok(team.id);
            }
        }

        Err(LinearError::InvalidInput {
            message: format!("Team with key '{}' not found", team_key),
        })
    }

    pub async fn resolve_user_id(&self, assignee: &str) -> Result<Option<String>> {
        if assignee == "me" {
            // Get current user from viewer query
            let viewer_data = self.execute_viewer_query().await?;
            Ok(Some(viewer_data.viewer.id))
        } else {
            // For now, return as-is (could be a UUID or email)
            // In later phases we'll add user search functionality
            Ok(Some(assignee.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use secrecy::SecretString;

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
        let _client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();
        // Client creation succeeded if we reach this point without panicking
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "invalid_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Authentication failed"));
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::builder()
            .auth_token(SecretString::new(api_key))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(api_key))
            .build()
            .expect("Failed to create client");
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();

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
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

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
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

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
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();

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
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .base_url(Some(server.url()))
            .build()
            .unwrap();
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

        let client = LinearClient::builder()
            .auth_token(SecretString::new(api_key))
            .build()
            .expect("Failed to create client");

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
