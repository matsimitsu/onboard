use anyhow::Result;

use crate::session::SshSession;
use crate::ui;

pub async fn execute(
    session: &SshSession,
    repo: &str,
    dest: &str,
    branch: Option<&str>,
) -> Result<()> {
    let branch = branch.unwrap_or("main");

    // For SSH repos, ensure the host key is in known_hosts
    if let Some(host) = ssh_host(repo) {
        let check = format!("grep -q '{}' ~/.ssh/known_hosts 2>/dev/null", host);
        if !session.run_check(&check).await? {
            ui::output(&format!("adding {} to known_hosts", host));
            session
                .run(&format!(
                    "mkdir -p ~/.ssh && ssh-keyscan {} >> ~/.ssh/known_hosts 2>/dev/null",
                    host
                ))
                .await?;
        }
    }

    // If the directory already exists with a .git, pull instead of clone
    if session.run_check(&format!("test -d {}/.git", dest)).await? {
        ui::output("already cloned, pulling latest");
        session
            .run(&format!("cd {} && git pull origin {}", dest, branch))
            .await
    } else {
        session
            .run(&format!("git clone --branch {} {} {}", branch, repo, dest))
            .await
    }
}

/// Extract the hostname from an SSH git URL.
fn ssh_host(repo: &str) -> Option<&str> {
    if let Some(rest) = repo.strip_prefix("ssh://") {
        rest.split('@').nth(1)?.split('/').next()
    } else if repo.contains('@') && repo.contains(':') {
        Some(repo.split('@').nth(1)?.split(':').next()?)
    } else {
        None
    }
}
