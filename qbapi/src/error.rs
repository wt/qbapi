use std::fmt::Display;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("API Error: {0}")]
    APIError(APIError),
    #[error(transparent)]
    OAuthError(#[from] openid::error::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid redirect URI: {0}")]
    InvalidRedirectUri(String),
}

#[derive(Debug, Error, Deserialize)]
pub struct APIError {
    #[serde(rename = "Error")]
    pub errors: Vec<APIErrorData>,
    pub r#type: String,
}

impl Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} with errors:", self.r#type)?;
        for e in &self.errors {
            write!(f, "\n* {}", e)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct APIErrorData {
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Detail")]
    pub detail: String,
    pub code: String,
}

impl Display for APIErrorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.detail)?;
        Ok(())
    }
}
