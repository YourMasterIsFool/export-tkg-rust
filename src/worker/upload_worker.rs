use aws_config::imds::client;
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;

use crate::core::s3_config::s3_config;

pub async fn upload_worker(ouput_file: &str) -> Result<(), aws_sdk_s3::Error> {
    let s3 = s3_config().await.expect("cannot upload in in s3");

    let bucket = "s3-bucket.api-jobseeker.site";
    let file_path = "donwload.xlsx";

    let body = ByteStream::from_path(Path::new("output.xlsx"))
        .await
        .expect("cannot open file output xlsx");

    let resp = s3
        .put_object()
        .bucket(bucket)
        .key(file_path)
        .body(body)
        .send()
        .await?;

    println!("Upload berhasil: {:?}", resp.e_tag());
    Ok(())
}
