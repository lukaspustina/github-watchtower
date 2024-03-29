use crate::{
    errors::*,
    github::{
        link::Links, AuthorizedClient, OAuthToken, Repository, GITHUB_ACCEPT_HEADER,
        GITHUB_LINK_HEADER_NAME,
    },
    utils::http::GeneralErrHandler,
};

use chrono::{DateTime, FixedOffset};
use failure::Fail;
use log::{debug, trace};
use reqwest::{self, header, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Sha(String);

impl Sha {
    pub fn new<T: Into<String>>(sha: T) -> Sha {
        Sha(sha.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: Sha,
    pub commit: CommitDetail,
}

#[derive(Debug, Deserialize)]
pub struct CommitDetail {
    pub author: PersonDetails,
    pub committer: PersonDetails,
    pub message: String,
    pub verification: Verification,
}

#[derive(Debug, Deserialize)]
pub struct PersonDetails {
    pub name: String,
    pub email: String,
    pub date: DateTime<FixedOffset>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    ExpiredKey,
    NotSigningKey,
    GpgverifyError,
    GpgverifyUnavailable,
    Unsigned,
    UnknownSignatureType,
    NoUser,
    UnverifiedEmail,
    BadEmail,
    UnknownKey,
    MalformedSignature,
    Invalid,
    Valid,
}

#[derive(Debug, Deserialize)]
pub struct Verification {
    pub verified: bool,
    pub reason: Reason,
    pub signature: Option<String>,
    pub payload: Option<String>,
}

/// Parameters to filter the commits returned by GitHub
///
/// See https://developer.github.com/v3/repos/commits/
/// Attention: from means "all commits until this"; cf. GitHub documentation, Parameters, "sha"
#[derive(Debug, Default, Serialize)]
pub struct Params {
    from: Option<Sha>,
    to: Option<Sha>,
    path: Option<String>,
    author: Option<String>,
    since: Option<DateTime<FixedOffset>>,
    until: Option<DateTime<FixedOffset>>,
}

impl Params {
    pub fn new() -> Params {
        Default::default()
    }

    pub fn from(self, from: Sha) -> Params {
        Params {
            from: from.into(),
            ..self
        }
    }

    pub fn to(self, to: Sha) -> Params {
        Params {
            to: to.into(),
            ..self
        }
    }

    pub fn path(self, path: String) -> Params {
        Params {
            path: path.into(),
            ..self
        }
    }

    pub fn author(self, author: String) -> Params {
        Params {
            author: author.into(),
            ..self
        }
    }

    pub fn since(self, since: DateTime<FixedOffset>) -> Params {
        Params {
            since: since.into(),
            ..self
        }
    }

    pub fn until(self, until: DateTime<FixedOffset>) -> Params {
        Params {
            until: until.into(),
            ..self
        }
    }
}

#[allow(clippy::implicit_hasher)]
impl From<&Params> for HashMap<&'static str, String> {
    fn from(p: &Params) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();

        if let Some(ref sha) = p.from {
            let Sha(sha) = sha;
            map.insert("sha", sha.clone());
        }
        if let Some(ref path) = p.path {
            map.insert("path", path.clone());
        }
        if let Some(ref author) = p.author {
            map.insert("author", author.clone());
        }
        if let Some(ref since) = p.since {
            map.insert("since", since.to_string());
        }
        if let Some(ref until) = p.until {
            map.insert("until", until.to_string());
        }

        map
    }
}

/// Get Commits -- ATTENTION: Currently paging is not supported
pub(crate) fn commits<T: Into<Option<Params>>>(
    client: &AuthorizedClient,
    repository: &Repository,
    params: T,
) -> Result<Vec<Commit>> {
    let params_opt: Option<_> = params.into();
    let commits = do_commits(client, repository, params_opt.as_ref())?;
    let commits = filter_to(commits, params_opt);

    Ok(commits)
}

/// Filter commits if `to` is set else just return passed commits
fn filter_to(commits: Vec<Commit>, params_opt: Option<Params>) -> Vec<Commit> {
    if let Some(p) = params_opt {
        if let Some(to) = p.to {
            let mut v = Vec::new();
            for c in commits.into_iter() {
                if c.sha == to {
                    v.push(c);
                    break;
                } else {
                    v.push(c);
                }
            }
            return v;
        }
    }
    commits
}

fn do_commits(
    client: &AuthorizedClient,
    repository: &Repository,
    params: Option<&Params>,
) -> Result<Vec<Commit>> {
    let query_params: Option<HashMap<_, _>> = params.map(From::from);
    let OAuthToken(ref token) = client.oauth_token;

    let url = format!(
        "https://api.github.com/repos/{owner}/{repository}/commits",
        owner = repository.owner,
        repository = repository.name
    );

    let mut commits: Vec<Commit> = Vec::new();

    let mut response = get_commits(&client, &url, query_params.as_ref(), &token)?;
    loop {
        let result: Vec<Commit> = response.json().map_err(|e| {
            e.context(ErrorKind::FailedToProcessHttpResponse(
                response.status(),
                "reading body".to_string(),
            ))
        })?;
        commits.extend(result);

        if let Some(next_link) = next_link(&response)? {
            trace!("Following next header: '{}'", next_link);
            response = get_commits(&client, next_link, query_params.as_ref(), &token)?;
        } else {
            break;
        }
    }

    Ok(commits)
}

fn get_commits(
    client: &AuthorizedClient,
    url: &str,
    query_params: Option<&HashMap<&'static str, String>>,
    token: &str,
) -> Result<Response> {
    let request = client
        .http
        .get(url)
        .query(&query_params)
        .header(header::ACCEPT, GITHUB_ACCEPT_HEADER)
        .bearer_auth(token);
    debug!("Request: '{:#?}'", request);

    let response: Response = request
        .send()
        .map_err(|e| e.context(ErrorKind::HttpRequestFailed))?
        .general_err_handler(StatusCode::OK)?;
    debug!("Response: '{:#?}'", response);

    Ok(response)
}

fn next_link(response: &Response) -> Result<Option<&str>> {
    if let Some(link_header_value) = response.headers().get(GITHUB_LINK_HEADER_NAME) {
        let value_str = link_header_value.to_str().map_err(|e| {
            e.context(ErrorKind::FailedToProcessHttpResponse(
                response.status(),
                "reading Link header".to_string(),
            ))
        })?;
        return Links::try_from(value_str).map(|l| l.next).map_err(|e| {
            Error::from(ErrorKind::FailedToProcessHttpResponse(response.status(), e))
        });
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;

    use serde_json;
    use spectral::prelude::*;

    #[test]
    fn deserialize_commits() {
        test::init();

        let endpoints_json = r#"
            [
                {
                    "url": "https://api.github.com/repos/octocat/Hello-World/commits/6dcb09b5b57875f334f61aebed695e2e4193db5e",
                    "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e",
                    "node_id": "MDY6Q29tbWl0NmRjYjA5YjViNTc4NzVmMzM0ZjYxYWViZWQ2OTVlMmU0MTkzZGI1ZQ==",
                    "html_url": "https://github.com/octocat/Hello-World/commit/6dcb09b5b57875f334f61aebed695e2e4193db5e",
                    "comments_url": "https://api.github.com/repos/octocat/Hello-World/commits/6dcb09b5b57875f334f61aebed695e2e4193db5e/comments",
                    "commit": {
                    "url": "https://api.github.com/repos/octocat/Hello-World/git/commits/6dcb09b5b57875f334f61aebed695e2e4193db5e",
                    "author": {
                        "name": "Monalisa Octocat",
                        "email": "support@github.com",
                        "date": "2011-04-14T16:00:49Z"
                    },
                    "committer": {
                        "name": "Monalisa Octocat",
                        "email": "support@github.com",
                        "date": "2011-04-14T16:00:49Z"
                    },
                    "message": "Fix all the bugs",
                    "tree": {
                        "url": "https://api.github.com/repos/octocat/Hello-World/tree/6dcb09b5b57875f334f61aebed695e2e4193db5e",
                        "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"
                    },
                    "comment_count": 0,
                    "verification": {
                        "verified": false,
                        "reason": "unsigned",
                        "signature": null,
                        "payload": null
                    }
                    },
                    "author": {
                    "login": "octocat",
                    "id": 1,
                    "node_id": "MDQ6VXNlcjE=",
                    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/users/octocat",
                    "html_url": "https://github.com/octocat",
                    "followers_url": "https://api.github.com/users/octocat/followers",
                    "following_url": "https://api.github.com/users/octocat/following{/other_user}",
                    "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
                    "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
                    "organizations_url": "https://api.github.com/users/octocat/orgs",
                    "repos_url": "https://api.github.com/users/octocat/repos",
                    "events_url": "https://api.github.com/users/octocat/events{/privacy}",
                    "received_events_url": "https://api.github.com/users/octocat/received_events",
                    "type": "User",
                    "site_admin": false
                    },
                    "committer": {
                    "login": "octocat",
                    "id": 1,
                    "node_id": "MDQ6VXNlcjE=",
                    "avatar_url": "https://github.com/images/error/octocat_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/users/octocat",
                    "html_url": "https://github.com/octocat",
                    "followers_url": "https://api.github.com/users/octocat/followers",
                    "following_url": "https://api.github.com/users/octocat/following{/other_user}",
                    "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
                    "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
                    "organizations_url": "https://api.github.com/users/octocat/orgs",
                    "repos_url": "https://api.github.com/users/octocat/repos",
                    "events_url": "https://api.github.com/users/octocat/events{/privacy}",
                    "received_events_url": "https://api.github.com/users/octocat/received_events",
                    "type": "User",
                    "site_admin": false
                    },
                    "parents": [
                    {
                        "url": "https://api.github.com/repos/octocat/Hello-World/commits/6dcb09b5b57875f334f61aebed695e2e4193db5e",
                        "sha": "6dcb09b5b57875f334f61aebed695e2e4193db5e"
                    }
                    ]
                }
            ]
        "#;

        let endpoints: ::std::result::Result<Vec<Commit>, _> = serde_json::from_str(endpoints_json);

        assert_that(&endpoints).is_ok().has_length(1);
    }

    #[test]
    fn deserialize_commits_with_verification_object() {
        test::init();

        let endpoints_json = r#"
[
{
  "sha": "72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
  "node_id": "MDY6Q29tbWl0MTkzNjY2NzI3OjcyY2Y2ZGY3M2RiZDFhMTNhYzA5NjMxOWUwMGNiNjNlMGYyODQ2Yzc=",
  "commit": {
    "author": {
      "name": "Lukas Pustina",
      "email": "lukas@pustina.de",
      "date": "2019-06-25T08:37:21Z"
    },
    "committer": {
      "name": "Lukas Pustina",
      "email": "lukas@pustina.de",
      "date": "2019-06-25T10:27:51Z"
    },
    "message": "Add travis config",
    "tree": {
      "sha": "ea7435f6d72196332c436474a42aea8ce030d424",
      "url": "https://api.github.com/repos/lukaspustina/github-watchtower/git/trees/ea7435f6d72196332c436474a42aea8ce030d424"
    },
    "url": "https://api.github.com/repos/lukaspustina/github-watchtower/git/commits/72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
    "comment_count": 0,
    "verification": {
      "verified": true,
      "reason": "valid",
      "signature": "-----BEGIN PGP SIGNATURE-----\nComment: GPGTools - http://gpgtools.org\n\niQIzBAABCAAdFiEEQWEMJmhTxtV/4Zdg7PtdAy2CkRIFAl0R9ysACgkQ7PtdAy2C\nkRKdzQ//cDyI9JX93+c/893g8TDLAIYyoLqbBL700wSjXEMO7WLkXYOJtFMO8jlA\nKjecVo+v2b0Eq7t8xAWrGPXGYyCdrbqIJg6eQRWaSkrS9PwIwrWcraPcduvWPHk2\n7bxCykiuXe+R01+00zMICZY0P0WnvuaoZo4kL7s6etgGY3sQff+fXUI8sGg8KN1Y\nav+t+bGKJnONa+BomLuIMNUuh29DaDytB2N/xuvhE3Pj/WEiYDDlhh3Wka7nTmsM\nxMhaK8+Jjjsv9rhzW63yPKrc4tHLUHLjvs3f8bPZbSgZqvS6YpY2/Nm7l20N4HBy\nxwUQ1Ee6YaE6GS6InXUEcoLZu0DxvOP476r1VZ/l6t2YTkcvYp7yi1zHIF3AuVQs\nA9gb4gK0aI7uyKrbT86XJCKAeu1CuOIpp6fGwD39maD1LgB6tYoIiFj8kOHxM0cp\nlCRdM+rF5Sgmr5UYaaEpFM6uWvQ7O7SJWn4j1FwQN6Ul++1CUQjoq8XczXQhZ9e0\n7bzOF+KlahNUWElxCiatiBsKGAhZEVzHp4LALJQE5s7X/Ea1fqkF+c87+0FQXGUT\nV5YwhHK6LTutfgxVqyCUlK3pshFxyEkHb2zKQsoIr02KWbZH8uTzs56xNHCJ6mI/\nANFLOdKLkRWNBARGMAuiM2hTyEUUOL0F9uSQMMzRQTlrkL3lWRA=\n=ivRW\n-----END PGP SIGNATURE-----",
      "payload": "tree ea7435f6d72196332c436474a42aea8ce030d424\nparent c255ad2347d00cae3dd2d7a21e1357e50413fc4f\nauthor Lukas Pustina <lukas@pustina.de> 1561451841 +0200\ncommitter Lukas Pustina <lukas@pustina.de> 1561458471 +0200\n\nAdd travis config\n"
    }
  },
  "url": "https://api.github.com/repos/lukaspustina/github-watchtower/commits/72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
  "html_url": "https://github.com/lukaspustina/github-watchtower/commit/72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
  "comments_url": "https://api.github.com/repos/lukaspustina/github-watchtower/commits/72cf6df73dbd1a13ac096319e00cb63e0f2846c7/comments",
  "author": {
    "login": "lukaspustina",
    "id": 398967,
    "node_id": "MDQ6VXNlcjM5ODk2Nw==",
    "avatar_url": "https://avatars0.githubusercontent.com/u/398967?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/lukaspustina",
    "html_url": "https://github.com/lukaspustina",
    "followers_url": "https://api.github.com/users/lukaspustina/followers",
    "following_url": "https://api.github.com/users/lukaspustina/following{/other_user}",
    "gists_url": "https://api.github.com/users/lukaspustina/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/lukaspustina/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/lukaspustina/subscriptions",
    "organizations_url": "https://api.github.com/users/lukaspustina/orgs",
    "repos_url": "https://api.github.com/users/lukaspustina/repos",
    "events_url": "https://api.github.com/users/lukaspustina/events{/privacy}",
    "received_events_url": "https://api.github.com/users/lukaspustina/received_events",
    "type": "User",
    "site_admin": false
  },
  "committer": {
    "login": "lukaspustina",
    "id": 398967,
    "node_id": "MDQ6VXNlcjM5ODk2Nw==",
    "avatar_url": "https://avatars0.githubusercontent.com/u/398967?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/lukaspustina",
    "html_url": "https://github.com/lukaspustina",
    "followers_url": "https://api.github.com/users/lukaspustina/followers",
    "following_url": "https://api.github.com/users/lukaspustina/following{/other_user}",
    "gists_url": "https://api.github.com/users/lukaspustina/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/lukaspustina/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/lukaspustina/subscriptions",
    "organizations_url": "https://api.github.com/users/lukaspustina/orgs",
    "repos_url": "https://api.github.com/users/lukaspustina/repos",
    "events_url": "https://api.github.com/users/lukaspustina/events{/privacy}",
    "received_events_url": "https://api.github.com/users/lukaspustina/received_events",
    "type": "User",
    "site_admin": false
  },
  "parents": [
    {
      "sha": "c255ad2347d00cae3dd2d7a21e1357e50413fc4f",
      "url": "https://api.github.com/repos/lukaspustina/github-watchtower/commits/c255ad2347d00cae3dd2d7a21e1357e50413fc4f",
      "html_url": "https://github.com/lukaspustina/github-watchtower/commit/c255ad2347d00cae3dd2d7a21e1357e50413fc4f"
    }
  ],
  "stats": {
    "total": 83,
    "additions": 74,
    "deletions": 9
  },
  "files": [
    {
      "sha": "73fbe4844016a6a952db4edea9daeb6cef092ee7",
      "filename": ".travis.yml",
      "status": "added",
      "additions": 53,
      "deletions": 0,
      "changes": 53,
      "blob_url": "https://github.com/lukaspustina/github-watchtower/blob/72cf6df73dbd1a13ac096319e00cb63e0f2846c7/.travis.yml",
      "raw_url": "https://github.com/lukaspustina/github-watchtower/raw/72cf6df73dbd1a13ac096319e00cb63e0f2846c7/.travis.yml",
      "contents_url": "https://api.github.com/repos/lukaspustina/github-watchtower/contents/.travis.yml?ref=72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
      "patch": "@@ -0,0 +1,53 @@\n+language: rust\n+services: docker\n+sudo: required\n+\n+matrix:\n+  include:\n+    # Linux\n+    - env: TARGET=x86_64-unknown-linux-gnu\n+      os: linux\n+      dist: bionic\n+      rust: 1.35.0\n+    - env: TARGET=x86_64-unknown-linux-gnu\n+      os: linux\n+      dist: bionic\n+      rust: stable\n+\n+    # Linux -- static binary\n+    - env: TARGET=x86_64-unknown-linux-musl\n+      os: linux\n+      dist: bionic\n+      rust: stable\n+      addons:\n+        apt:\n+          packages:\n+          - musl\n+          - musl-dev\n+          - musl-tools\n+\n+    # macOS\n+    - env: TARGET=x86_64-apple-darwin\n+      os: osx\n+      rust: stable\n+\n+    # Testing for future releases of Rust\n+    - env: TARGET=x86_64-unknown-linux-gnu\n+      os: linux\n+      dist: bionic\n+      rust: beta\n+    - env: TARGET=x86_64-unknown-linux-gnu\n+      os: linux\n+      dist: bionic\n+      rust: nightly\n+\n+  allow_failures:\n+    - rust: beta\n+    - rust: nightly\n+\n+before_script:\n+  make _install\n+\n+script:\n+  make\n+"
    },
    {
      "sha": "9fb7651a82185c0153d38b6df700925c403bcf66",
      "filename": "Makefile",
      "status": "modified",
      "additions": 21,
      "deletions": 9,
      "changes": 30,
      "blob_url": "https://github.com/lukaspustina/github-watchtower/blob/72cf6df73dbd1a13ac096319e00cb63e0f2846c7/Makefile",
      "raw_url": "https://github.com/lukaspustina/github-watchtower/raw/72cf6df73dbd1a13ac096319e00cb63e0f2846c7/Makefile",
      "contents_url": "https://api.github.com/repos/lukaspustina/github-watchtower/contents/Makefile?ref=72cf6df73dbd1a13ac096319e00cb63e0f2846c7",
      "patch": "@@ -1,16 +1,24 @@\n+ifdef TARGET\n+\tTARGET_ARG=--target $(TARGET)\n+else\n+\tTARGET_ARG=\n+endif\n+\n all: check build test clippy fmt-check\n \n+$(info TARGET_ARG=\"$(TARGET_ARG)\")\n+\n todos:\n \trg --vimgrep -g '!Makefile' -i todo \n \n check:\n-\tcargo check --all --tests --examples\n+\tcargo check $(TARGET_ARG) --all --tests --examples\n \n build:\n-\tcargo build --all --tests --examples\n+\tcargo build $(TARGET_ARG) --all --tests --examples\n \n test:\n-\tcargo test\n+\tcargo test $(TARGET_ARG)\n \n clean-package:\n \tcargo clean -p $$(cargo read-manifest | jq -r .name)\n@@ -27,14 +35,18 @@ fmt-check:\n duplicate_libs:\n \tcargo tree -d\n \n-_update-clippy_n_fmt:\n-\trustup update\n-\trustup component add clippy\n-\trustup component add rustfmt --toolchain=nightly\n-\n _cargo_install:\n \tcargo install -f cargo-tree\n \tcargo install -f cargo-bump\n \n-.PHONY: tests\n+_install:\n+\t@if test $$TARGET; then \\\n+\t\techo \"Adding rust target $(TARGET)\"; \\\n+\t\trustup target add $(TARGET); \\\n+\tfi\n+\trustup component add clippy\n+\trustup toolchain install nightly\n+\trustup component add rustfmt --toolchain=nightly\n+\n+.PHONY: \n "
    }
  ]
}
]
        "#;

        let endpoints: ::std::result::Result<Vec<Commit>, _> = serde_json::from_str(endpoints_json);

        assert_that(&endpoints).is_ok().has_length(1);
    }
}
