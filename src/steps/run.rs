use anyhow::Result;

use crate::session::SshSession;
use crate::ui;

pub async fn execute(
    session: &SshSession,
    command: &str,
    working_dir: Option<&str>,
    screen: Option<&str>,
    unless: Option<&str>,
) -> Result<()> {
    // Skip if the `unless` check passes
    if let Some(check) = unless
        && session.run_check(check).await?
    {
        ui::output("skipped (already done)");
        return Ok(());
    }

    let mut cmd = String::new();

    if let Some(dir) = working_dir {
        cmd.push_str(&format!("cd {} && ", dir));
    }

    cmd.push_str(command);

    if let Some(name) = screen {
        // Skip if a screen session with this name is already running
        let check = format!("screen -list | grep -q '\\.{}\\b'", name);
        if session.run_check(&check).await? {
            ui::output("skipped (screen session already running)");
            return Ok(());
        }

        let wrapped = format!("screen -dmS {} bash -c 'source ~/.bashrc && {}'", name, cmd);
        session.run(&wrapped).await
    } else {
        session.run(&cmd).await
    }
}
