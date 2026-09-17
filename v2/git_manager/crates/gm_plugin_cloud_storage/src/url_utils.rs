/// Cloud Storage URL generation and parsing utilities.
///
/// Supports:
/// - S3: https://s3.amazonaws.com/bucket/repo.git
/// - GCS: https://storage.googleapis.com/bucket/repo.git
/// - Azure Blob: https://bucket.blob.core.windows.net/repo.git

/// Information parsed from a cloud storage repository URL.
#[derive(Debug, Clone)]
pub struct CloudStorageUrl {
    pub provider:  String,
    pub bucket:    String,
    pub repo:      String,
    pub full_name: String,
}

/// Generates a cloud storage clone URL.
pub fn generate_url(provider: &str, bucket: &str, repo: &str) -> String {
    match provider {
        "s3"    => format!("https://s3.amazonaws.com/{bucket}/{repo}.git"),
        "gcs"   => format!("https://storage.googleapis.com/{bucket}/{repo}.git"),
        "azure" => format!("https://{bucket}.blob.core.windows.net/{repo}.git"),
        _       => format!("https://{bucket}/{repo}.git"),
    }
}

/// Parses a cloud storage URL into its components.
pub fn parse_url(url: &str) -> Result<CloudStorageUrl, String> {
    let trimmed = url.trim_end_matches(".git").trim_end_matches('/');

    // S3: https://s3.amazonaws.com/bucket/repo
    if let Some(path) = trimmed.strip_prefix("https://s3.amazonaws.com/") {
        if let Some((bucket, repo)) = path.split_once('/') {
            if !bucket.is_empty() && !repo.is_empty() && !repo.contains('/') {
                return Ok(CloudStorageUrl {
                    provider: "s3".to_string(),
                    bucket: bucket.to_string(),
                    repo: repo.to_string(),
                    full_name: format!("{bucket}/{repo}"),
                });
            }
        }
    }

    // GCS: https://storage.googleapis.com/bucket/repo
    if let Some(path) = trimmed.strip_prefix("https://storage.googleapis.com/") {
        if let Some((bucket, repo)) = path.split_once('/') {
            if !bucket.is_empty() && !repo.is_empty() && !repo.contains('/') {
                return Ok(CloudStorageUrl {
                    provider: "gcs".to_string(),
                    bucket: bucket.to_string(),
                    repo: repo.to_string(),
                    full_name: format!("{bucket}/{repo}"),
                });
            }
        }
    }

    // Azure: https://bucket.blob.core.windows.net/repo
    if let Some(rest) = trimmed.strip_prefix("https://") {
        if let Some((bucket_part, repo)) = rest.split_once('/') {
            if !repo.is_empty() && !repo.contains('/') {
                if let Some(bucket) = bucket_part.strip_suffix(".blob.core.windows.net") {
                    if !bucket.is_empty() {
                        return Ok(CloudStorageUrl {
                            provider: "azure".to_string(),
                            bucket: bucket.to_string(),
                            repo: repo.to_string(),
                            full_name: format!("{bucket}/{repo}"),
                        });
                    }
                }
            }
        }
    }

    Err(format!("Invalid cloud storage URL: {url}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s3_url_generation() {
        assert_eq!(
            generate_url("s3", "my-bucket", "my-repo"),
            "https://s3.amazonaws.com/my-bucket/my-repo.git"
        );
    }

    #[test]
    fn gcs_url_generation() {
        assert_eq!(
            generate_url("gcs", "my-bucket", "my-repo"),
            "https://storage.googleapis.com/my-bucket/my-repo.git"
        );
    }

    #[test]
    fn azure_url_generation() {
        assert_eq!(
            generate_url("azure", "my-bucket", "my-repo"),
            "https://my-bucket.blob.core.windows.net/my-repo.git"
        );
    }

    #[test]
    fn s3_url_parsing() {
        let parsed = parse_url("https://s3.amazonaws.com/my-bucket/my-repo.git").unwrap();
        assert_eq!(parsed.provider, "s3");
        assert_eq!(parsed.bucket, "my-bucket");
        assert_eq!(parsed.repo, "my-repo");
    }
}
