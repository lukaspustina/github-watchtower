use crate::{
    errors::*,
    github::{AuthorizedClient, OAuthToken},
    utils::http::GeneralErrHandler,
};

use failure::Fail;
use log::debug;
use reqwest::{self, header, Response, StatusCode};
use std::collections::HashMap;

pub type Endpoints = HashMap<String, String>;

pub(crate) fn endpoints(client: &AuthorizedClient) -> Result<Endpoints> {
    let OAuthToken(ref token) = client.oauth_token;
    let request = client
        .http
        .get("https://api.github.com/")
        .header(
            header::ACCEPT,
            "Accept: application/vnd.github.v3+json".as_bytes(),
        )
        .bearer_auth(token);
    debug!("Request: '{:#?}'", request);

    let mut response: Response = request
        .send()
        .map_err(|e| e.context(ErrorKind::HttpRequestFailed))?
        .general_err_handler(StatusCode::OK)?;
    debug!("Response: '{:#?}'", response);

    let result = response.json().map_err(|e| {
        e.context(ErrorKind::FailedToProcessHttpResponse(
            response.status(),
            "reading body".to_string(),
        ))
    })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;

    use serde_json;
    use spectral::prelude::*;

    #[test]
    fn deserialize_endpoints() {
        test::init();

        let endpoints_json = r#"
            {
                "current_user_url": "https://api.github.com/user",
                "current_user_authorizations_html_url": "https://github.com/settings/connections/applications{/client_id}",
                "authorizations_url": "https://api.github.com/authorizations",
                "code_search_url": "https://api.github.com/search/code?q={query}{&page,per_page,sort,order}",
                "commit_search_url": "https://api.github.com/search/commits?q={query}{&page,per_page,sort,order}",
                "emails_url": "https://api.github.com/user/emails",
                "emojis_url": "https://api.github.com/emojis",
                "events_url": "https://api.github.com/events",
                "feeds_url": "https://api.github.com/feeds",
                "followers_url": "https://api.github.com/user/followers",
                "following_url": "https://api.github.com/user/following{/target}",
                "gists_url": "https://api.github.com/gists{/gist_id}",
                "hub_url": "https://api.github.com/hub",
                "issue_search_url": "https://api.github.com/search/issues?q={query}{&page,per_page,sort,order}",
                "issues_url": "https://api.github.com/issues",
                "keys_url": "https://api.github.com/user/keys",
                "notifications_url": "https://api.github.com/notifications",
                "organization_repositories_url": "https://api.github.com/orgs/{org}/repos{?type,page,per_page,sort}",
                "organization_url": "https://api.github.com/orgs/{org}",
                "public_gists_url": "https://api.github.com/gists/public",
                "rate_limit_url": "https://api.github.com/rate_limit",
                "repository_url": "https://api.github.com/repos/{owner}/{repo}",
                "repository_search_url": "https://api.github.com/search/repositories?q={query}{&page,per_page,sort,order}",
                "current_user_repositories_url": "https://api.github.com/user/repos{?type,page,per_page,sort}",
                "starred_url": "https://api.github.com/user/starred{/owner}{/repo}",
                "starred_gists_url": "https://api.github.com/gists/starred",
                "team_url": "https://api.github.com/teams",
                "user_url": "https://api.github.com/users/{user}",
                "user_organizations_url": "https://api.github.com/user/orgs",
                "user_repositories_url": "https://api.github.com/users/{user}/repos{?type,page,per_page,sort}",
                "user_search_url": "https://api.github.com/search/users?q={query}{&page,per_page,sort,order}"
            }
        "#;

        let endpoints: ::std::result::Result<Endpoints, _> = serde_json::from_str(endpoints_json);

        assert_that(&endpoints).is_ok().has_length(31);
    }
}
