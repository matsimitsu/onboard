use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct SshSession {
    host: String,
    socket: PathBuf,
    _tempdir: tempfile::TempDir,
}

impl SshSession {
    pub async fn connect(host: &str) -> Result<Self> {
        let tempdir = tempfile::tempdir().context("Failed to create temp dir")?;
        let socket = tempdir.path().join("ctl");

        // Start ControlMaster with agent forwarding
        let status = Command::new("ssh")
            .arg("-A")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg(format!("ControlPath={}", socket.display()))
            .arg("-o")
            .arg("ControlMaster=yes")
            .arg("-o")
            .arg("ControlPersist=yes")
            .arg("-M")
            .arg("-f")
            .arg("-N")
            .arg(host)
            .status()
            .await
            .with_context(|| format!("Failed to start SSH master to {}", host))?;

        if !status.success() {
            anyhow::bail!("SSH master connection to {} failed", host);
        }

        Ok(Self {
            host: host.to_string(),
            socket,
            _tempdir: tempdir,
        })
    }

    fn ssh_cmd(&self) -> Command {
        let mut cmd = Command::new("ssh");
        cmd.arg("-o")
            .arg(format!("ControlPath={}", self.socket.display()))
            .arg("-o")
            .arg("ControlMaster=no")
            .arg(&self.host);
        cmd
    }

    /// Run a command remotely, streaming output through ui::output.
    pub async fn run(&self, cmd: &str) -> Result<()> {
        let mut child = self
            .ssh_cmd()
            .arg("--")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute: {}", cmd))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let h1 = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                crate::ui::output(&line);
            }
        });

        let h2 = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                crate::ui::output(&line);
            }
        });

        let _ = h1.await;
        let _ = h2.await;

        let status = child
            .wait()
            .await
            .with_context(|| format!("Failed to wait for: {}", cmd))?;

        if !status.success() {
            anyhow::bail!("Command failed (exit {}): {}", status, cmd);
        }
        Ok(())
    }

    /// Run a command and capture its stdout (no terminal output).
    #[allow(dead_code)]
    pub async fn run_output(&self, cmd: &str) -> Result<String> {
        let output = self
            .ssh_cmd()
            .arg("--")
            .arg(cmd)
            .output()
            .await
            .with_context(|| format!("Failed to execute: {}", cmd))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Command failed (exit {}): {}\n{}",
                output.status,
                cmd,
                stderr
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Run a command and return whether it succeeded (exit code 0). No output.
    pub async fn run_check(&self, cmd: &str) -> Result<bool> {
        let output = self
            .ssh_cmd()
            .arg("--")
            .arg(cmd)
            .output()
            .await
            .with_context(|| format!("Failed to execute check: {}", cmd))?;

        Ok(output.status.success())
    }

    /// Upload a local file to a remote path using scp via the control socket.
    pub async fn upload(&self, local: &Path, remote: &str) -> Result<()> {
        // Ensure parent directory exists on remote
        if let Some((dir, _)) = remote.rsplit_once('/') {
            self.run(&format!("mkdir -p {}", dir)).await.ok();
        }

        let output = Command::new("scp")
            .arg("-q")
            .arg("-o")
            .arg(format!("ControlPath={}", self.socket.display()))
            .arg(local.to_str().context("Invalid local path")?)
            .arg(format!("{}:{}", self.host, remote))
            .output()
            .await
            .context("Failed to run scp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "scp failed: {} -> {}: {}",
                local.display(),
                remote,
                stderr.trim()
            );
        }
        Ok(())
    }

    /// Upload string content directly to a remote path.
    pub async fn upload_content(&self, content: &str, remote: &str) -> Result<()> {
        let tmp = tempfile::NamedTempFile::new()?;
        tokio::fs::write(tmp.path(), content).await?;
        self.upload(tmp.path(), remote).await
    }

    pub async fn close(self) -> Result<()> {
        Command::new("ssh")
            .arg("-o")
            .arg(format!("ControlPath={}", self.socket.display()))
            .arg("-O")
            .arg("exit")
            .arg(&self.host)
            .output()
            .await
            .ok();
        Ok(())
    }
}
