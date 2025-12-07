//! Bun JS Challenge Provider

use crate::provider::JsChallengeError;
use std::io::Write;
use std::process::{Command, Stdio};

/// Bun-based JavaScript Challenge Provider
pub struct BunJCP {
    code: String,
}

impl BunJCP {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
        }
    }

    pub fn solve(&self, func_name: &str, challenge: &str) -> Result<String, JsChallengeError> {
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
            return Err(JsChallengeError::Runtime(format!(
                "Bun execution failed: {}",
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
