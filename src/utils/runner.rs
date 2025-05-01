use anyhow::Result;
use std::process::Command;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub execution_time: Duration,
}

pub fn compile_cpp(source_file: &str) -> Result<bool> {
    let output = Command::new("g++")
        .args(&[
            "-std=c++17",
            "-O2",
            source_file,
            "-o",
            &format!("{}.out", source_file),
        ])
        .output()?;

    Ok(output.status.success())
}

pub fn run_with_input(executable: &str, input: &str, time_limit: Duration) -> Result<RunResult> {
    let start = Instant::now();

    let mut child = Command::new(executable)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes())?;
    }

    // Convert Duration to milliseconds for wait_timeout_ms
    let timeout_ms = time_limit.as_millis() as u32;
    match child.wait_timeout_ms(timeout_ms)? {
        Some(status) => {
            let output = child.wait_with_output()?;
            let execution_time = start.elapsed();

            Ok(RunResult {
                success: status.success(),
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                execution_time,
            })
        }
        None => {
            child.kill()?;
            Ok(RunResult {
                success: false,
                output: "Time limit exceeded".to_string(),
                execution_time: time_limit,
            })
        }
    }
}
