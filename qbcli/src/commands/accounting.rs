use anyhow::{Context, Result};
use directories::ProjectDirs;
use oo7::Keyring;
use qbapi::error::{APIError, Error};
use reqwest::Client;
use serde_json::{Map, Value};
use tracing::{error, info};
use url::Url;

use crate::{
    config::{get_oauth_creds, read_config_data_from_config_file},
    util::{
        auth::{QBAuthData, get_stored_profile_auth_token},
        cli::ProfileArgs,
    },
};

#[derive(Debug, clap::Args)]
pub(crate) struct AccountingArgs {
    #[command(flatten)]
    profile_args: ProfileArgs,

    #[command(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, clap::Subcommand)]
enum SubCommands {
    /// Login and get access token for Quickbooks Online API
    CompanyInfo,
    /// Generic query; supports many data types
    Query(QueryArgs),
}

#[derive(Debug, clap::Parser)]
struct QueryArgs {
    query: String,
}

pub async fn do_accounting(
    accounting_args: &AccountingArgs,
    project_dirs: &ProjectDirs,
) -> Result<()> {
    let config = read_config_data_from_config_file(&project_dirs)?;
    let profile = accounting_args.profile_args.profile(&config);
    let keyring = Keyring::new().await?;
    let mut qb_auth_data = get_stored_profile_auth_token(&keyring, profile)
        .await?
        .ok_or(anyhow::anyhow!(
            "Profile doesn't exist in the secret storage."
        ))?;
    let app_creds = get_oauth_creds(&qb_auth_data.environment)?;
    let api_client = qb_auth_data.environment.api_client();
    let oauth_client = api_client
        .oauth_client_without_redirect(&app_creds.client_id, &app_creds.client_secret)
        .await?;
    qb_auth_data.refresh_token(&oauth_client).await?;
    let base_url = api_client.base_url;

    let access_token = qb_auth_data.token.bearer.access_token.as_str();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", access_token).as_str().parse().unwrap(),
    );
    headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    match accounting_args.subcommand {
        SubCommands::CompanyInfo => do_company_info(&qb_auth_data, &client, &base_url).await?,
        SubCommands::Query(ref query_args) => {
            do_query(query_args, &qb_auth_data, &client, &base_url).await?
        }
    }
    Ok(())
}

pub async fn do_company_info(
    qb_auth_data: &QBAuthData,
    client: &Client,
    base_url: &Url,
) -> Result<()> {
    let url = base_url.join(
        format!(
            "{}{}/companyinfo/{}",
            qbapi::accounting::BASE_URL_FRAGMENT,
            &qb_auth_data.realm,
            &qb_auth_data.realm
        )
        .as_ref(),
    )?;
    info!("url to fetch: {url}");

    let resp = client
        .get(Url::parse(url.as_str()).unwrap())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let bytes = resp.bytes().await?;
    let body_json: Value = serde_json::from_slice(&bytes)?;
    println!("{}", serde_json::to_string_pretty(&body_json)?);

    Ok(())
}

async fn do_query(
    query_args: &QueryArgs,
    qb_auth_data: &QBAuthData,
    client: &Client,
    base_url: &Url,
) -> Result<()> {
    let mut url = base_url.join(
        format!(
            "{}{}/query",
            qbapi::accounting::BASE_URL_FRAGMENT,
            &qb_auth_data.realm
        )
        .as_ref(),
    )?;
    url.query_pairs_mut()
        .append_pair("query", &query_args.query);
    info!("url to fetch: {url}");

    let resp = client
        .get(Url::parse(url.as_str()).unwrap())
        .send()
        .await
        .unwrap();

    match resp.error_for_status_ref() {
        Ok(_) => {
            let bytes = resp
                .bytes()
                .await
                .context("Cannot get body from error response.")?;
            let body_json: Value =
                serde_json::from_slice(&bytes).context("Can't parse body as JSON.")?;
            println!("{}", serde_json::to_string_pretty(&body_json)?);
        }
        Err(e) => {
            let bytes = resp
                .bytes()
                .await
                .context("Cannot get body from error response.")?;
            let mut body_json: Map<String, Value> = serde_json::from_slice(&bytes)
                .context("Cannot decode json from error response body")?;
            let v = body_json
                .remove("Fault")
                .context("Error response with no fault information")?;
            let error: APIError = serde_json::from_value(v)
                .context("Cannot deserialize API error details from response")?;
            error!("e: {e:#?}");
            return Err(anyhow::anyhow!(Error::APIError(error)));
        }
    };
    Ok(())
}
