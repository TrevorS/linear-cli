// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug"
)]
pub struct Viewer;

pub fn hello() -> &'static str {
    "Linear SDK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Linear SDK");
    }

    #[test]
    fn test_viewer_query_builds() {
        // This test verifies that the GraphQL code generation worked
        let _query = Viewer::build_query(viewer::Variables {});
    }
}
