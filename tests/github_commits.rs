use github_watchtower::github::{
    Client,
    GitHub,
    OAuthToken,
    Repository,
};

use env_logger;
use spectral::prelude::*;
use std::env;

#[test]
#[ignore]
fn github_commits() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let repository = Repository::new("lukaspustina", "github-watchtower");
    let endpoints = client.commits(&repository);

    asserting("there are commits").that(&endpoints).is_ok().matches(|x| x.len() > 0);
}

