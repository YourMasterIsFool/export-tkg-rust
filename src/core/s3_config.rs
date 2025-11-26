use std::env;

use aws_config::Region;
use aws_sdk_s3::{Client, config::Config, config::Credentials};
use dotenv::dotenv;

#[allow(dead_code)]
pub async fn s3_config() -> Result<Client, aws_sdk_s3::Error> {
    dotenv().ok();
    let access_key = env::var("AWS_ACCESS_KEY_ID").unwrap();
    let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let bucket = env::var("AWS_BUCKET_NAME").unwrap();

    let credential = Credentials::new(access_key, secret_key, None, None, "manual".into());

    let config = Config::builder()
        .region(Region::new(region))
        .credentials_provider(credential)
        .behavior_version_latest()
        .build();

    let client = aws_sdk_s3::Client::from_conf(config);

    Ok(client)
}
