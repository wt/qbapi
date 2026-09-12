const SANDBOX_DISC_URL: &str =
    "https://developer.api.intuit.com/.well-known/openid_sandbox_configuration";
// const PRODUCTION_DISC_URL: &str =
//     "https://developer.api.intuit.com/.well-known/openid_configuration";

pub async fn qb_sandbox_provider_config() -> Result<openid::Config, crate::error::Error> {
    let client = reqwest::Client::new();
    let resp = client
        .get(SANDBOX_DISC_URL)
        .send()
        .await?
        .error_for_status()?;
    let config: openid::Config = resp.json().await.unwrap();
    Ok(config)
}
