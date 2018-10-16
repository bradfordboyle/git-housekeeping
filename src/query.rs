use std::env;

use failure;
use graphql_client::{GraphQLQuery, Response};
use reqwest;

use self::my_query::RustBranchViewRepositoryRefsNodes;
use self::my_query::RustBranchViewRepositoryRefsNodesTargetOn::Commit;

type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query.graphql",
    response_derives = "Clone,Debug",
)]
struct MyQuery;

#[derive(Debug, Serialize)]
pub struct BranchInfo {
    name: String,
    // TODO use a real DateTime object here
    when: String,
    author: String,
}

impl BranchInfo {
    fn from(node: RustBranchViewRepositoryRefsNodes) -> Result<BranchInfo, failure::Error> {
        let name = node.name;

        // TODO use a result type
        match node.target.on {
            Commit(commit) => Ok(BranchInfo {
                name: name,
                when: commit.committed_date,
                author: commit
                    .author
                    .ok_or(failure::err_msg("missing author"))?
                    .name
                    .ok_or(failure::err_msg("no author name"))?,
            }),
            _ => Err(failure::err_msg("expecting a Commit")),
        }
    }
}

pub fn perform_my_query(owner: &str, name: &str) -> Result<Vec<BranchInfo>, failure::Error> {
    let mut variables = my_query::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
        cursor: None,
        fetch_size: Some(10),
    };

    let mut branch_vec = Vec::new();

    let api_token = env::var("GITHUB_API_TOKEN")?;

    let client = reqwest::Client::new();
    loop {
        // this is the important line
        let request_body = MyQuery::build_query(variables.clone());
        let mut res = client
            .post("https://api.github.com/graphql")
            .bearer_auth(api_token.clone())
            .json(&request_body)
            .send()?;

        let response_body: Response<my_query::ResponseData> = res.json()?;

        let response_data = response_body
            .data
            .ok_or(failure::err_msg("no response data"))?;
        let refs = response_data
            .repository
            .ok_or(failure::err_msg("missing repository"))?
            .refs
            .ok_or(failure::err_msg("missing refs"))?;
        let page_info = refs.page_info;

        let nodes = refs.nodes.ok_or(failure::err_msg("missing nodes"))?;

        for node in nodes {
            if let Some(node) = node {
                branch_vec.push(BranchInfo::from(node)?);
            }
        }

        variables.cursor = page_info.end_cursor;
        if !page_info.has_next_page {
            break;
        }
    }

    Ok(branch_vec)
}
