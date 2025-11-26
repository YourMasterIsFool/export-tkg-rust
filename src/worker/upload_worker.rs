use crate::core::s3_config::s3_config;
use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};
use dotenv::dotenv;
use std::{env, path::Path, time::Duration};
use uuid::Uuid;

#[allow(dead_code)]
pub async fn upload_worker(ouput_file: &str) -> Result<String, aws_sdk_s3::Error> {
    dotenv().ok();
    let s3 = s3_config().await.expect("cannot upload in in s3");

    // let bucket = "s3-bucket.api-jobseeker.site";
    let region = env::var("AWS_REGION").unwrap();
    let bucket = env::var("AWS_BUCKET_NAME").unwrap();
    let id = Uuid::new_v4();
    let file_path = format!("tkg-export/{}-{}.xlsx", ouput_file, id);

    let body = ByteStream::from_path(Path::new("output.xlsx"))
        .await
        .map_err(|e| println!("cannot open file output xlsx {}", e));

    let upload_s3 = s3
        .put_object()
        .bucket(&bucket)
        .key(&file_path)
        .body(body.unwrap())
        .send()
        .await
        .map_err(|err| println!("error upload s3 {}", err));

    let presign_cfg = PresigningConfig::expires_in(Duration::from_hours(42))
        .map_err(|err| println!("failed setting Presigning {}", err))
        .unwrap();
    let presigned = s3
        .get_object()
        .bucket(&bucket)
        .key(&file_path)
        .presigned(presign_cfg)
        .await
        .map_err(|err| println!("error presigning aws {}", err))
        .unwrap();

    // let url = s3_public_url(&bucket, &region, &file_path);
    Ok(presigned.uri().to_string())
}

// pub fn s3_public_url(bucket: &str, region: &str, key: &str) -> String {
//     format!("https://{bucket}.s3.{region}.amazonaws.com/{key}")
// }
