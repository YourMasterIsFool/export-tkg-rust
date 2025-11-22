use aws_config::Region;
use aws_sdk_s3::Client;

pub async fn s3_config() -> Result<Client, aws_sdk_s3::Error> {
    let config = aws_config::from_env()
        .region(Region::new("ap-southeast-3"))
        .load()
        .await;

    let client = Client::new(&config);
    let client = aws_sdk_s3::Client::new(&config);

    Ok(client)
}
