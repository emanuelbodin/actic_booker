use aws_config::BehaviorVersion;
use aws_sdk_ssm::Client as SsmClient;
use lambda_runtime::Error;
use std::env;

pub async fn get_credentials(user: &str) -> Result<(String, String), Error> {
    if let (Ok(username), Ok(password)) = (env::var("USERNAME"), env::var("PASSWORD")) {
        if !username.is_empty() && !password.is_empty() {
            return Ok((username, password));
        }
    }

    let prefix = env::var("SSM_PARAMETER_PREFIX").unwrap_or_else(|_| "/actic-booker".to_string());
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = SsmClient::new(&config);

    let username = get_parameter(&client, &format!("{prefix}/{user}/username")).await?;
    let password = get_parameter(&client, &format!("{prefix}/{user}/password")).await?;

    Ok((username, password))
}

async fn get_parameter(client: &SsmClient, name: &str) -> Result<String, Error> {
    let value = client
        .get_parameter()
        .name(name)
        .with_decryption(true)
        .send()
        .await
        .map_err(|err| Error::from(format!("failed to read SSM parameter {name}: {err}")))?
        .parameter()
        .and_then(|parameter| parameter.value().map(str::to_string))
        .ok_or_else(|| Error::from(format!("SSM parameter {name} has no value")))?;

    Ok(value)
}
