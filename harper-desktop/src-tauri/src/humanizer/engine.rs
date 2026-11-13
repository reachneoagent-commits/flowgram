//! Runs the bundled llama.cpp server for one model and streams rewrites from it.
//!
//! The model file carries its own chat template and sampling defaults, so we send the draft as a
//! plain chat message and let the server build the exact prompt the model was trained on.

use super::models::Tier;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};

/// Longest draft one rewrite accepts. The engine's context is 8,192 tokens and the output needs
/// room too; longer text is split into sections by the caller.
pub const MAX_INPUT_CHARS: usize = 9_000;
const CONTEXT_TOKENS: &str = "8192";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("The rewriting engine isn't installed with this copy of Gramflow.")]
    NoBinary,
    #[error("That model isn't downloaded yet.")]
    NoModel,
    #[error("The rewriting engine didn't start: {0}")]
    Start(String),
    #[error("The rewriting engine stopped responding: {0}")]
    Request(String),
    #[error("That text is too long to rewrite in one go. Select a shorter passage.")]
    TooLong,
    #[error("Nothing to rewrite.")]
    Empty,
    #[error("The rewrite was cancelled.")]
    Cancelled,
}

pub struct Engine {
    child: Child,
    port: u16,
    token: String,
    pub tier: Tier,
    last_used: Instant,
}

/// Finds `llama-server` in the app's bundled resources, or at `GRAMFLOW_LLAMA_SERVER` when
/// developing.
pub fn find_binary(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "llama-server.exe"
    } else {
        "llama-server"
    };
    if let Some(path) = std::env::var_os("GRAMFLOW_LLAMA_SERVER").map(PathBuf::from)
        && path.is_file()
    {
        return Some(path);
    }
    let path = resource_dir?.join("engine").join(name);
    path.is_file().then_some(path)
}

fn free_port() -> std::io::Result<u16> {
    Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?
        .local_addr()?
        .port())
}

impl Engine {
    pub async fn start(binary: &Path, tier: Tier) -> Result<Engine, EngineError> {
        let model = tier
            .path()
            .filter(|_| tier.is_installed())
            .ok_or(EngineError::NoModel)?;
        Self::start_with_model(binary, &model, tier).await
    }

    /// Starts the engine on a specific model file. Updates use this to try a downloaded model
    /// before it replaces the one in use.
    pub async fn start_with_model(
        binary: &Path,
        model: &Path,
        tier: Tier,
    ) -> Result<Engine, EngineError> {
        let port = free_port().map_err(|error| EngineError::Start(error.to_string()))?;
        // Other programs on this computer can reach localhost ports, so require a secret.
        let token: String = (0..32).map(|_| fastrand::alphanumeric()).collect();

        let mut command = Command::new(binary);
        command
            .args(["-m"])
            .arg(model)
            .args(["--host", "127.0.0.1", "--port"])
            .arg(port.to_string())
            .args(["--api-key", &token])
            .args(["-c", CONTEXT_TOKENS, "-np", "1", "-ngl", "99", "--no-webui"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if let Some(dir) = binary.parent() {
            command.current_dir(dir);
        }
        #[cfg(windows)]
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW

        let child = command
            .spawn()
            .map_err(|error| EngineError::Start(error.to_string()))?;

        let mut engine = Engine {
            child,
            port,
            token,
            tier,
            last_used: Instant::now(),
        };
        engine.wait_until_ready().await?;
        Ok(engine)
    }

    async fn wait_until_ready(&mut self) -> Result<(), EngineError> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let started = Instant::now();
        loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Err(EngineError::Start(format!("it exited with {status}")));
            }
            if client
                .get(&url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                return Ok(());
            }
            if started.elapsed() > STARTUP_TIMEOUT {
                return Err(EngineError::Start(
                    "it took too long to load the model".into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn idle_for(&self) -> Duration {
        self.last_used.elapsed()
    }

    /// Streams the rewrite, calling `on_text` with each new piece, and returns the full text.
    pub async fn rewrite(
        &mut self,
        draft: &str,
        cancel: Arc<AtomicBool>,
        mut on_text: impl FnMut(&str),
    ) -> Result<String, EngineError> {
        let draft = draft.trim();
        if draft.is_empty() {
            return Err(EngineError::Empty);
        }
        if draft.chars().count() > MAX_INPUT_CHARS {
            return Err(EngineError::TooLong);
        }
        self.last_used = Instant::now();

        let max_tokens = output_budget(draft);
        let response = reqwest::Client::new()
            .post(format!(
                "http://127.0.0.1:{}/v1/chat/completions",
                self.port
            ))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({
                "messages": [{ "role": "user", "content": draft }],
                "max_tokens": max_tokens,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|error| EngineError::Request(error.to_string()))?;
        if !response.status().is_success() {
            return Err(EngineError::Request(format!(
                "status {}",
                response.status()
            )));
        }

        let mut full = String::new();
        let mut pending = String::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if cancel.load(Ordering::Relaxed) {
                return Err(EngineError::Cancelled);
            }
            let chunk = chunk.map_err(|error| EngineError::Request(error.to_string()))?;
            pending.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(newline) = pending.find('\n') {
                let line: String = pending.drain(..=newline).collect();
                if let Some(piece) = parse_sse_line(&line) {
                    full.push_str(&piece);
                    on_text(&piece);
                }
            }
        }
        self.last_used = Instant::now();
        Ok(full.trim().to_string())
    }
}

/// About 2.5 times the draft's token count, within the model's recommended 256 to 2,048.
fn output_budget(draft: &str) -> usize {
    let estimated_tokens = draft.chars().count() / 3 + 1;
    (estimated_tokens * 5 / 2).clamp(256, 2048)
}

/// Returns the text of one server-sent-events line, or `None` for anything else.
fn parse_sse_line(line: &str) -> Option<String> {
    let data = line.trim().strip_prefix("data:")?.trim();
    if data == "[DONE]" {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(data).ok()?;
    let piece = value["choices"][0]["delta"]["content"].as_str()?;
    (!piece.is_empty()).then(|| piece.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_content_delta() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_sse_line(line).as_deref(), Some("Hello"));
    }

    #[test]
    fn ignores_done_empty_and_non_data_lines() {
        assert_eq!(parse_sse_line("data: [DONE]"), None);
        assert_eq!(parse_sse_line(""), None);
        assert_eq!(parse_sse_line(": keep-alive"), None);
        assert_eq!(parse_sse_line(r#"data: {"choices":[{"delta":{}}]}"#), None);
        assert_eq!(
            parse_sse_line(r#"data: {"choices":[{"delta":{"content":""}}]}"#),
            None
        );
        assert_eq!(parse_sse_line("data: not json"), None);
    }

    #[test]
    fn output_budget_stays_within_bounds() {
        assert_eq!(output_budget("hi"), 256);
        assert_eq!(output_budget(&"a".repeat(MAX_INPUT_CHARS)), 2048);
        let mid = output_budget(&"a".repeat(900));
        assert!((256..2048).contains(&mid));
    }

    #[test]
    fn find_binary_returns_none_without_a_bundle() {
        assert_eq!(find_binary(None), None);
        assert_eq!(find_binary(Some(Path::new("/nonexistent"))), None);
    }

    /// End-to-end check against a real engine and model. Run it by hand:
    /// `GRAMFLOW_LLAMA_SERVER=/path/to/llama-server cargo test live_rewrite -- --ignored`
    #[tokio::test]
    #[ignore = "needs llama-server and a downloaded model"]
    async fn live_rewrite_streams_and_keeps_numbers() {
        let binary = find_binary(None).expect("set GRAMFLOW_LLAMA_SERVER");
        let tier = crate::humanizer::models::TIERS
            .iter()
            .map(|info| info.tier)
            .find(|tier| tier.is_installed())
            .expect("download a model first");
        let mut engine = Engine::start(&binary, tier).await.unwrap();

        let draft = "Thanks for reaching out. I wanted to circle back and leverage our previous discussion to ensure alignment going forward. The budget is $180K and the deadline is March 3.";
        let mut pieces = 0;
        let text = engine
            .rewrite(draft, Arc::new(AtomicBool::new(false)), |_| pieces += 1)
            .await
            .unwrap();

        assert!(pieces > 1, "the rewrite should stream in several pieces");
        assert!(!text.is_empty());
        assert!(
            crate::humanizer::numbers::check(draft, &text).is_clean(),
            "{text}"
        );
        assert!(matches!(
            engine
                .rewrite("   ", Arc::new(AtomicBool::new(false)), |_| {})
                .await,
            Err(EngineError::Empty)
        ));
    }
}
