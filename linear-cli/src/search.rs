// ABOUTME: Search functionality for the Linear CLI application
// ABOUTME: Handles query parsing, result formatting, and search operations

use linear_sdk::{LinearClient, Result, SearchResult};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub issues_only: bool,
    pub docs_only: bool,
    pub projects_only: bool,
    pub limit: i32,
    pub include_archived: bool,
}

impl SearchOptions {
    #[allow(dead_code)]
    pub fn new(query: String) -> Self {
        Self {
            query,
            issues_only: false,
            docs_only: false,
            projects_only: false,
            limit: 10,
            include_archived: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub exclusions: Vec<String>,
    pub field_searches: std::collections::HashMap<String, String>,
    pub exact_phrases: Vec<String>,
}

impl SearchQuery {
    pub fn parse(query: &str) -> Self {
        let mut result = SearchQuery {
            text: String::new(),
            exclusions: Vec::new(),
            field_searches: std::collections::HashMap::new(),
            exact_phrases: Vec::new(),
        };

        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut escape_next = false;

        for ch in query.chars() {
            if escape_next {
                current_token.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                }
                '"' => {
                    if in_quotes {
                        result.exact_phrases.push(current_token.clone());
                        current_token.clear();
                        in_quotes = false;
                    } else {
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                        in_quotes = true;
                    }
                }
                ' ' | '\t' if !in_quotes => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            if in_quotes {
                result.exact_phrases.push(current_token);
            } else {
                tokens.push(current_token);
            }
        }

        // Process tokens
        let mut text_parts = Vec::new();
        for token in tokens {
            if token.starts_with('-') && token.len() > 1 {
                // Exclusion
                result.exclusions.push(token[1..].to_string());
            } else if token.contains(':') {
                // Field search
                let parts: Vec<&str> = token.splitn(2, ':').collect();
                if parts.len() == 2 {
                    result
                        .field_searches
                        .insert(parts[0].to_string(), parts[1].to_string());
                } else {
                    text_parts.push(token);
                }
            } else {
                text_parts.push(token);
            }
        }

        result.text = text_parts.join(" ");
        result
    }
}

pub async fn search(client: &LinearClient, options: SearchOptions) -> Result<SearchResult> {
    let parsed_query = SearchQuery::parse(&options.query);

    // For now, implement basic text search
    // In future iterations, we'll add field search and exclusion support
    let search_text = if !parsed_query.text.is_empty() {
        parsed_query.text
    } else if !parsed_query.exact_phrases.is_empty() {
        parsed_query.exact_phrases.join(" ")
    } else {
        options.query // Fallback to original query
    };

    // Determine which searches to perform based on options
    let search_all = !options.issues_only && !options.docs_only && !options.projects_only;

    let mut result = SearchResult {
        issues: Vec::new(),
        documents: Vec::new(),
        projects: Vec::new(),
    };

    if search_all || options.issues_only {
        result.issues = client
            .search_issues(&search_text, options.limit, options.include_archived)
            .await?;
    }

    if search_all || options.docs_only {
        result.documents = client
            .search_documents(&search_text, options.limit, options.include_archived)
            .await?;
    }

    // Projects search would be implemented here when available
    // if search_all || options.projects_only {
    //     result.projects = client.search_projects(&search_text, options.limit, options.include_archived).await?;
    // }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_query() {
        let query = SearchQuery::parse("login bug");
        assert_eq!(query.text, "login bug");
        assert!(query.exclusions.is_empty());
        assert!(query.field_searches.is_empty());
        assert!(query.exact_phrases.is_empty());
    }

    #[test]
    fn test_parse_exact_phrase() {
        let query = SearchQuery::parse("\"login system\" other");
        assert_eq!(query.text, "other");
        assert_eq!(query.exact_phrases, vec!["login system"]);
        assert!(query.exclusions.is_empty());
        assert!(query.field_searches.is_empty());
    }

    #[test]
    fn test_parse_exclusions() {
        let query = SearchQuery::parse("login -mobile -tablet");
        assert_eq!(query.text, "login");
        assert_eq!(query.exclusions, vec!["mobile", "tablet"]);
        assert!(query.field_searches.is_empty());
        assert!(query.exact_phrases.is_empty());
    }

    #[test]
    fn test_parse_field_searches() {
        let query = SearchQuery::parse("assignee:john team:ENG status:todo");
        assert_eq!(query.text, "");
        assert_eq!(
            query.field_searches.get("assignee"),
            Some(&"john".to_string())
        );
        assert_eq!(query.field_searches.get("team"), Some(&"ENG".to_string()));
        assert_eq!(
            query.field_searches.get("status"),
            Some(&"todo".to_string())
        );
    }

    #[test]
    fn test_parse_mixed_query() {
        let query = SearchQuery::parse("\"login bug\" assignee:john -mobile dashboard");
        assert_eq!(query.text, "dashboard");
        assert_eq!(query.exact_phrases, vec!["login bug"]);
        assert_eq!(query.exclusions, vec!["mobile"]);
        assert_eq!(
            query.field_searches.get("assignee"),
            Some(&"john".to_string())
        );
    }

    #[test]
    fn test_search_options_new() {
        let options = SearchOptions::new("test query".to_string());
        assert_eq!(options.query, "test query");
        assert!(!options.issues_only);
        assert!(!options.docs_only);
        assert!(!options.projects_only);
        assert_eq!(options.limit, 10);
        assert!(!options.include_archived);
    }
}
