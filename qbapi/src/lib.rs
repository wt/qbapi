use openid::{Client as OIDClient, DiscoveredClient};
use url::Url;

use crate::{auth::qb_sandbox_provider_config, error::Error};

pub mod accounting;
pub mod auth;
pub mod error;

pub static SANDBOX_BASE_URL: &str = "https://sandbox-quickbooks.api.intuit.com/";
pub static PRODUCTION_BASE_URL: &str = "https://quickbooks.api.intuit.com/";

pub enum Environment {
    Production,
    Sandbox,
}

pub struct Client {
    environment: Environment,
    pub base_url: Url,
}

impl Client {
    pub fn new(environment: Environment) -> Self {
        let base_url = match environment {
            Environment::Production => {
                Url::parse(PRODUCTION_BASE_URL).expect("Bad base production Url")
            }
            Environment::Sandbox => Url::parse(SANDBOX_BASE_URL).expect("Bad base sandbox Url"),
        }
        .clone();
        Self {
            environment,
            base_url,
        }
    }

    pub async fn oauth_client(
        &self,
        client_id: impl AsRef<str>,
        client_secret: impl AsRef<str>,
        redirect_listen_host: Option<&str>,
        redirect_listen_port: Option<u16>,
    ) -> Result<OIDClient, Error> {
        let client_id = client_id.as_ref();
        let client_secret = client_secret.as_ref();

        let redirect = match (redirect_listen_host, redirect_listen_port) {
            (Some(h), Some(p)) => Some(format!("https://{}:{}", h, p)),
            (None, None) => None,
            _ => {
                return Err(Error::InvalidRedirectUri(
                    "Redirect host:port invalid. Either both or neither should be provided."
                        .to_owned(),
                ));
            }
        };

        match self.environment {
            Environment::Production => Ok(DiscoveredClient::discover(
                client_id.to_owned(),
                client_secret.to_owned(),
                redirect,
                Url::parse("https://developer.api.intuit.com/").expect("Invalid url for issuer"),
            )
            .await?),
            Environment::Sandbox => {
                let http_client = reqwest::Client::new();
                let config = qb_sandbox_provider_config().await?;
                let jwks = openid::jwks(&http_client, config.jwks_uri.clone()).await?;
                let provider = config.into();

                Ok(openid::Client::new(
                    provider,
                    client_id.into(),
                    Some(client_secret.into()),
                    redirect,
                    reqwest::Client::new(),
                    Some(jwks),
                ))
            }
        }
    }

    pub async fn oauth_client_without_redirect(
        &self,
        client_id: impl AsRef<str>,
        client_secret: impl AsRef<str>,
    ) -> Result<OIDClient, Error> {
        let client_id = client_id.as_ref();
        let client_secret = client_secret.as_ref();

        match self.environment {
            Environment::Production => Ok(DiscoveredClient::discover(
                client_id.to_owned(),
                client_secret.to_owned(),
                None,
                Url::parse("https://developer.api.intuit.com/").expect("Invalid url for issuer"),
            )
            .await?),
            Environment::Sandbox => {
                let http_client = reqwest::Client::new();
                let config = qb_sandbox_provider_config().await?;
                let jwks = openid::jwks(&http_client, config.jwks_uri.clone()).await?;
                let provider = config.into();

                Ok(openid::Client::new(
                    provider,
                    client_id.into(),
                    Some(client_secret.into()),
                    None,
                    reqwest::Client::new(),
                    Some(jwks),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
