use aws_sdk_s3 as s3;

#[derive(Clone)]
pub struct S3Manager {
    s3_client: s3::Client,
    bucket_name: String,
}

impl S3Manager {
    pub async fn new(
        bucket_name: String,
        account_id: String,
        access_key_id: String,
        access_key_secret: String,
    ) -> Self {
        let config = aws_config::from_env()
            .endpoint_url(format!("https://{}.r2.cloudfarestorage.com", account_id))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                access_key_id,
                access_key_secret,
                None, // session token is not used with R2
                None, // doesn't expire
                "R2",
            ))
            .region("auto")
            .load()
            .await;

        let client = s3::Client::new(&config);

        Self {
            s3_client: client,
            bucket_name,
        }
    }
}
