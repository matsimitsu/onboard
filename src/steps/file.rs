use std::path::Path;

use anyhow::Result;

use crate::session::SshSession;

pub async fn execute(
    session: &SshSession,
    src: &str,
    dest: &str,
    mode: Option<&str>,
    owner: Option<&str>,
) -> Result<()> {
    session.upload(Path::new(src), dest).await?;

    if let Some(m) = mode {
        session.run(&format!("chmod {} {}", m, dest)).await?;
    }
    if let Some(o) = owner {
        session.run(&format!("sudo chown {} {}", o, dest)).await?;
    }

    Ok(())
}
