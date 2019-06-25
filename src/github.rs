use crate::errors::*;

use failure::Fail;
use log::debug;
use reqwest::{self, Response, StatusCode};

mod endpoints;

pub use endpoints::Endpoints;

#[derive(Debug)]
pub struct Client {}

impl Client {
    pub fn with_oauth_token(oauth_token: &OAuthToken) -> AuthorizedClient {
        AuthorizedClient{
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
pub struct OAuthToken(String);

pub trait GitHub {
    fn endpoints(&self) -> Result<Endpoints>;
}

impl<'a> GitHub for AuthorizedClient<'a> {
    fn endpoints(&self) -> Result<Endpoints> {
        endpoints::endpoints(self)
    }
}

