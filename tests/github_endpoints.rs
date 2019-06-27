use github_watchtower::github::{Client, GitHub, OAuthToken};

use env_logger;
use spectral::prelude::*;
use std::env;

#[test]
#[ignore]
fn github_endpoints() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let endpoints = client.endpoints();

    assert_that(&endpoints).is_ok().has_length(31);
}
