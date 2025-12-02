use super::JsRuntime;
use std::fs;
use std::process::{Command, Stdio};

/// Bun-based JavaScript solver (uses external bun process)
pub struct BunSolver {
    code: String,
}

impl JsRuntime for BunSolver {
    fn from_prepared(code: &str) -> Result<Self, String> {
        // Verify bun is available
        Command::new("bun")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| "Bun is not installed or not in PATH".to_string())?;

        Ok(Self {
            code: code.to_string(),
        })
    }

    fn solve_n(&self, challenge: &str) -> Result<String, String> {
        self.call_solver("n", challenge)
    }

    fn solve_sig(&self, challenge: &str) -> Result<String, String> {
        self.call_solver("sig", challenge)
    }

    fn has_n(&self) -> bool {
        true
    }

    fn has_sig(&self) -> bool {
        true
    }
}

impl BunSolver {
    fn call_solver(&self, func_name: &str, challenge: &str) -> Result<String, String> {
        let script = format!(
            r#"const _result = {{}};
{}
const result = _result.{}("{}");
console.log(result);
"#,
            self.code,
            func_name,
            challenge.replace('\\', "\\\\").replace('"', "\\\"")
        );

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("ejs_rs_bun_{}.js", std::process::id()));

        fs::write(&temp_file, &script).map_err(|e| format!("Failed to write temp file: {}", e))?;

        let output = Command::new("bun")
            .args(["run", temp_file.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run bun: {}", e))?;

        let _ = fs::remove_file(&temp_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Bun execution failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }
}
