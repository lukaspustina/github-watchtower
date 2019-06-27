use crate::errors::*;

use reqwest;

pub mod commits;
pub mod endpoints;

pub use commits::Commit;
pub use endpoints::Endpoints;

use commits::Params;

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub fn with_oauth_token(oauth_token: &OAuthToken) -> AuthorizedClient {
        AuthorizedClient {
            oauth_token,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Debug)]
pub struct AuthorizedClient<'a> {
    oauth_token: &'a OAuthToken,
    http: reqwest::Client,
}

#[derive(Debug)]
pub struct OAuthToken(pub String);

#[derive(Debug)]
pub struct Repository<'a> {
    owner: &'a str,
    name: &'a str,
}

impl<'a> Repository<'a> {
    pub fn new(owner: &'a str, name: &'a str) -> Repository<'a> {
        Repository { owner, name }
    }
}

pub trait GitHub {
    fn commits<T: Into<Option<Params>>>(
        &self,
        repository: &Repository,
        params: T,
    ) -> Result<Vec<Commit>>;
    fn endpoints(&self) -> Result<Endpoints>;
}

impl<'a> GitHub for AuthorizedClient<'a> {
    fn commits<T: Into<Option<Params>>>(
        &self,
        repository: &Repository,
        params: T,
    ) -> Result<Vec<Commit>> {
        commits::commits(self, repository, params)
    }

    fn endpoints(&self) -> Result<Endpoints> {
        endpoints::endpoints(self)
    }
}
