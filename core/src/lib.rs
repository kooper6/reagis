use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncReadExt;
use std::time::Duration;
use std::os::unix::process::CommandExt;

#[derive(Debug)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub struct CommandRunner {
    timeout: Duration,
    max_memory_bytes: Option<u64>,
    max_output_bytes: u64,
}

impl CommandRunner {
    pub fn new(timeout_secs: u64, max_memory_mb: Option<u64>, max_output_bytes: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            max_memory_bytes: max_memory_mb.map(|mb| mb * 1024 * 1024),
            max_output_bytes,
        }
    }

    pub async fn run_command(&self, program: &str, args: &[String]) -> Result<ExecutionResult, String> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mem_limit_opt = self.max_memory_bytes;

        unsafe {
            cmd.pre_exec(move || {
                if let Some(mem_limit) = mem_limit_opt {
                    let rlim = libc::rlimit {
                        rlim_cur: mem_limit as libc::rlim_t,
                        rlim_max: mem_limit as libc::rlim_t,
                    };

                    if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
                        //TODO fix error handling
                    }
                }
                Ok(())
            });
        }

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn command '{}': {}", program, e))?;

        let mut stdout = child.stdout.take().expect("Failed to open stdout");
        let mut stderr = child.stderr.take().expect("Failed to open stderr");

        let max_output = self.max_output_bytes;

        let execution_future = async {
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            // Limit output size to prevent OOM
            let mut stdout_limited = stdout.take(max_output);
            let mut stderr_limited = stderr.take(max_output);

            // Use tokio::try_join to read both streams concurrently
            let (read_stdout_res, read_stderr_res) = tokio::join!(
                stdout_limited.read_to_end(&mut stdout_buf),
                stderr_limited.read_to_end(&mut stderr_buf)
            );

            read_stdout_res.map_err(|e| format!("Failed to read stdout: {}", e))?;
            read_stderr_res.map_err(|e| format!("Failed to read stderr: {}", e))?;

            let status = child.wait().await
                .map_err(|e| format!("Failed to wait for child: {}", e))?;

            Ok::<_, String>((status, stdout_buf, stderr_buf))
        };

        match tokio::time::timeout(self.timeout, execution_future).await {
            Ok(Ok((status, stdout_bytes, stderr_bytes))) => {
                Ok(ExecutionResult {
                    stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
                    stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
                    exit_code: status.code(),
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                let _ = child.start_kill();
                Err(format!("Command '{}' timed out after {:?}", program, self.timeout))
            }
        }
    }
}