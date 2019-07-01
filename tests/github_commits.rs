use github_watchtower::github::{
    commits::{Params, Sha},
    Client, GitHub, OAuthToken, Repository,
};

use chrono::prelude::*;
use env_logger;
use log::debug;
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
    let commits = client.commits(&repository, None);

    asserting("there are commits")
        .that(&commits)
        .is_ok()
        .matches(|x| !x.is_empty());
}

#[test]
#[ignore]
fn github_commits_from_sha() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let repository = Repository::new("lukaspustina", "github-watchtower");

    let params = Params::new().from(Sha::new("10b1bf9f34fcab001615cb6a9fa7b3ca71d7d5ca"));
    let commits = client
        .commits(&repository, params)
        .expect("Failed to retrieve commits from '10b1bf9f34fcab001615cb6a9fa7b3ca71d7d5ca'");

    debug!("From: {:#?}", commits);

    let amount = commits.len();
    asserting("number of commit is 3")
        .that(&amount)
        .is_equal_to(&3);
}

#[test]
#[ignore]
fn github_commits_from_sha_to_sha() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let repository = Repository::new("lukaspustina", "github-watchtower");

    let params = Params::new()
        .from(Sha::new("a01d2a79a8a0e740e81ae2a0de31362219b33f50"))
        .to(Sha::new("10b1bf9f34fcab001615cb6a9fa7b3ca71d7d5ca"));
    let commits = client
        .commits(&repository, params)
        .expect("Failed to retrieve commits from '10b1bf9f34fcab001615cb6a9fa7b3ca71d7d5ca'");

    debug!("From: {:#?}", commits);

    let amount = commits.len();
    asserting("number of commit is 4")
        .that(&amount)
        .is_equal_to(&4);
}

#[test]
#[ignore]
fn github_commits_since() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let repository = Repository::new("lukaspustina", "github-watchtower");

    let all_commits = client
        .commits(&repository, None)
        .expect("Failed to retrieve all commits");
    let params = Params::new()
        .since(DateTime::parse_from_rfc2822("Wed, 26 Jun 2019 09:36:26 +0200").unwrap());
    let since_commits = client
        .commits(&repository, params)
        .expect("Failed to retrieve commits from 'Wed Jun 26 09:36:26 2019 +0200'");
    let diff = all_commits.len() - since_commits.len();

    asserting("number of commits is less than total number")
        .that(&diff)
        .is_equal_to(&4);
}

#[test]
#[ignore]
fn github_commits_until() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var_os("GITHUB_TOKEN")
        .expect("Environment variable 'GITHUB_TOKEN' is not set.")
        .to_string_lossy()
        .to_string();

    let token = OAuthToken(token);
    let client = Client::with_oauth_token(&token);

    let repository = Repository::new("lukaspustina", "github-watchtower");

    let params = Params::new()
        .until(DateTime::parse_from_rfc2822("Tue, 25 Jun 2019 14:56:32 +0200").unwrap());
    let commits = client
        .commits(&repository, params)
        .expect("Failed to retrieve commits from 'Wed Jun 26 09:36:26 2019 +0200'");
    let amount = commits.len();

    asserting("number of commits is less than total number")
        .that(&amount)
        .is_equal_to(&4);
}
