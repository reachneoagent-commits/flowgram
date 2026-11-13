//! Resumable model download with a checksum check.

use super::models::Tier;
use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub tier: &'static str,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Couldn't find a place to save the model.")]
    NoFolder,
    #[error("Couldn't reach the download server: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Couldn't save the model: {0}")]
    Disk(#[from] std::io::Error),
    #[error("The download server answered with an error ({0}).")]
    Status(u16),
    #[error("The download was cancelled.")]
    Cancelled,
    #[error("The downloaded file didn't match its checksum. It was removed; try again.")]
    ChecksumMismatch,
}

/// Downloads `tier`, resuming a partial file when one exists. Calls `on_progress` about every
/// 250 ms. Setting `cancel` stops the download and keeps the partial file for a later resume.
pub async fn download(
    tier: Tier,
    cancel: Arc<AtomicBool>,
    on_progress: impl FnMut(Progress),
) -> Result<(), DownloadError> {
    if tier.is_installed() {
        return Ok(());
    }
    let final_path = tier.path().ok_or(DownloadError::NoFolder)?;
    let part_path = tier.partial_path().ok_or(DownloadError::NoFolder)?;
    download_file(
        tier.info().id,
        &tier.url(),
        &part_path,
        &final_path,
        None,
        cancel,
        on_progress,
    )
    .await
}

/// Downloads `url` into `part_path`, resuming when a partial file exists, then moves it to
/// `final_path`. With `expected_sha` the result must match it exactly; without one, the server's
/// own checksum header is used when it sends one.
pub async fn download_file(
    label: &'static str,
    url: &str,
    part_path: &std::path::Path,
    final_path: &std::path::Path,
    expected_sha: Option<&str>,
    cancel: Arc<AtomicBool>,
    mut on_progress: impl FnMut(Progress),
) -> Result<(), DownloadError> {
    std::fs::create_dir_all(final_path.parent().ok_or(DownloadError::NoFolder)?)?;
    let part_path = part_path.to_path_buf();
    let final_path = final_path.to_path_buf();

    let client = reqwest::Client::builder()
        .user_agent("Gramflow")
        .connect_timeout(std::time::Duration::from_secs(20))
        .build()?;

    let resume_from = std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);
    let mut request = client.get(url);
    if resume_from > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
    }
    let response = request.send().await?;

    let status = response.status();
    // 416 means the partial file already holds everything; fall through to verification.
    let already_complete = status.as_u16() == 416;
    if !status.is_success() && !already_complete {
        return Err(DownloadError::Status(status.as_u16()));
    }

    // The server ignored the range and sent the whole file: start over.
    let resumed = status.as_u16() == 206;
    let start = if resumed { resume_from } else { 0 };
    let header_sha = response
        .headers()
        .get("x-linked-etag")
        .or_else(|| response.headers().get(reqwest::header::ETAG))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim_matches('"').to_ascii_lowercase())
        .filter(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()));
    let expected_sha = expected_sha
        .map(|sha| sha.to_ascii_lowercase())
        .or(header_sha);
    let total = start + response.content_length().unwrap_or(0);

    if !already_complete {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(&part_path)?;

        let mut downloaded = start;
        let mut last_report = std::time::Instant::now();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if cancel.load(Ordering::Relaxed) {
                file.flush()?;
                return Err(DownloadError::Cancelled);
            }
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            if last_report.elapsed() >= std::time::Duration::from_millis(250) {
                last_report = std::time::Instant::now();
                on_progress(Progress {
                    tier: label,
                    downloaded,
                    total,
                });
            }
        }
        file.flush()?;
        on_progress(Progress {
            tier: label,
            downloaded,
            total: downloaded,
        });
    }

    if let Some(expected) = expected_sha {
        let part = part_path.clone();
        let actual = tokio::task::spawn_blocking(move || sha256_of(&part))
            .await
            .map_err(|error| std::io::Error::other(error.to_string()))??;
        if actual != expected {
            let _ = std::fs::remove_file(&part_path);
            return Err(DownloadError::ChecksumMismatch);
        }
    }

    std::fs::rename(&part_path, &final_path)?;
    Ok(())
}

fn sha256_of(path: &std::path::Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_a_known_digest() {
        let dir = std::env::temp_dir().join("gramflow-humanizer-sha-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("abc.txt");
        std::fs::write(&path, b"abc").unwrap();

        assert_eq!(
            sha256_of(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_dir_all(dir).ok();
    }
}
