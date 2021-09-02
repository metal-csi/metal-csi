use super::*;
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdout, Command},
};

#[derive(Debug)]
pub struct LocalShell {
    pub sudo: bool,
    pub chroot: Option<String>,
}

#[async_trait]
impl ControlModuleTrait for LocalShell {
    async fn is_connected(&self) -> Result<bool> {
        Ok(true)
    }

    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn exec(&self, cmd: &str) -> Result<(String, u32)> {
        let cmd = self.build_command(self.sudo, self.chroot.as_ref().map(|s| s.as_str()), cmd);
        let result = Command::new("sh")
            .args(&["-c", &cmd])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output()
            .await?;

        Ok((
            format!(
                "{}\n{}",
                std::str::from_utf8(&result.stdout)?.trim_end(),
                std::str::from_utf8(&result.stderr)?.trim_end(),
            ),
            result.status.code().unwrap_or(256) as u32,
        ))
    }

    async fn exec_open(&self, cmd: &str) -> Result<ControlStream> {
        let cmd = self.build_command(self.sudo, self.chroot.as_ref().map(|s| s.as_str()), cmd);
        let mut child = Command::new("sh")
            .args(&["-c", &cmd])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr = BufReader::new(child.stderr.take().unwrap());

        Ok(ControlStream(Box::new(LocalShellStream {
            child: Some(child),
            stdout,
            stderr,
        })))
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct LocalShellStream {
    child: Option<Child>,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

#[async_trait]
impl ControlStreamTrait for LocalShellStream {
    async fn wait_for_completion(&mut self) -> Result<(String, u32)> {
        if let Some(child) = self.child.take() {
            let result = child.wait_with_output().await?;
            Ok((
                format!(
                    "{}\n{}",
                    std::str::from_utf8(&result.stdout)?.trim_end(),
                    std::str::from_utf8(&result.stderr)?.trim_end(),
                ),
                result.status.code().unwrap_or(256) as u32,
            ))
        } else {
            Err(AppError::Generic(format!(
                "This stream has been completed!"
            )))
        }
    }

    async fn wait_for(&mut self, ptrn: &Regex) -> Result<(String, Option<u32>)> {
        if let Some(child) = &mut self.child {
            let mut output = String::new();
            let mut found = false;
            let mut code = None;
            while !found && code.is_none() {
                let mut stdout_line = Default::default();
                let mut stderr_line = Default::default();
                tokio::select! {
                    val = self.stdout.read_line(&mut stdout_line) => {
                        if val? == 0 {
                            continue;
                        }
                    }
                    val = self.stderr.read_line(&mut stderr_line) => {
                        if val? == 0 {
                            continue;
                        }
                    }
                    val = child.wait() => {
                        code = val?.code().map(|v| v as u32);
                    }
                }
                debug!("{}{}", stdout_line, stderr_line);
                if ptrn.is_match(&stdout_line) || ptrn.is_match(&stderr_line) {
                    found = true;
                }
                output.push_str(&stdout_line);
                output.push_str(&stderr_line);
            }
            Ok((output, code))
        } else {
            Err(AppError::Generic("Child process unavailable!".into()))
        }
    }

    async fn sendline(&mut self, data: &str) -> Result<()> {
        if let Some(child) = &mut self.child {
            if let Some(stdin) = &mut child.stdin {
                stdin.write(data.as_bytes()).await?;
                return Ok(());
            }
        }
        Ok(())
    }
}
