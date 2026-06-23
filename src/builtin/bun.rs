//! Bun JS Challenge Provider

use crate::provider::JsChallengeError;
use crate::trace::{debug, error};
use std::io::Write;
use std::process::{Command, Stdio};

/// Bun-based JavaScript Challenge Provider
pub struct BunJCP {
    code: String,
}

impl BunJCP {
    pub fn new(code: &str) -> Self {
        debug!(code_len = code.len(), "Creating Bun provider");
        Self {
            code: code.to_string(),
        }
    }

    pub fn solve(&self, func_name: &str, challenge: &str) -> Result<String, JsChallengeError> {
        debug!(%func_name, %challenge, script_len = self.code.len(), "Calling solver via Bun");
        let escaped = challenge.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "const _result = {{}};\n{}\nconsole.log(_result.{}(\"{}\"));",
            self.code, func_name, escaped
        );

        let mut child = Command::new("bun")
            .args(["run", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(%stderr, exit_code = ?output.status.code(), "Bun execution failed");
            return Err(JsChallengeError::Runtime(format!(
                "Bun execution failed: {}",
                stderr.trim()
            )));
        }

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!(%func_name, result_len = result.len(), result, exit_code = ?output.status.code(), "Bun solver returned");
        Ok(result)
    }
}
