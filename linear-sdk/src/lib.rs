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