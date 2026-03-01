use std::collections::HashMap;

use anyhow::Result;

use crate::config::Settings;
use crate::session::SshSession;
use crate::ui;

pub async fn execute(
    session: &SshSession,
    vars: &HashMap<String, String>,
    settings: &Settings,
) -> Result<()> {
    let profile = &settings.shell_profile;

    for (key, value) in vars {
        let line = format!("export {}=\"{}\"", key, value);
        let check = format!("grep -qF '{}' {}", line, profile);
        if !session.run_check(&check).await? {
            session
                .run(&format!("echo '{}' >> {}", line, profile))
                .await?;
            ui::output(&format!("set {} in {}", key, profile));
        } else {
            ui::output(&format!("{} already set in {}", key, profile));
        }
    }

    Ok(())
}
