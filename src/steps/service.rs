use std::collections::HashMap;

use anyhow::Result;

use crate::session::SshSession;

pub async fn execute(
    session: &SshSession,
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    restart: Option<&str>,
    env: Option<&HashMap<String, String>>,
) -> Result<()> {
    let unit = generate_unit_file(name, command, working_dir, restart, env);
    let unit_path = format!("/etc/systemd/system/{}.service", name);

    session.upload_content(&unit, &unit_path).await?;

    session.run("sudo systemctl daemon-reload").await?;
    session
        .run(&format!("sudo systemctl enable {}", name))
        .await?;
    session
        .run(&format!("sudo systemctl restart {}", name))
        .await?;

    Ok(())
}

fn generate_unit_file(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    restart: Option<&str>,
    env: Option<&HashMap<String, String>>,
) -> String {
    let mut unit = format!(
        "[Unit]\nDescription={}\nAfter=network.target\n\n[Service]\nExecStart={}\n",
        name, command
    );

    if let Some(wd) = working_dir {
        unit.push_str(&format!("WorkingDirectory={}\n", wd));
    }

    unit.push_str(&format!("Restart={}\n", restart.unwrap_or("on-failure")));

    if let Some(env_vars) = env {
        for (k, v) in env_vars {
            unit.push_str(&format!("Environment=\"{}={}\"\n", k, v));
        }
    }

    unit.push_str("\n[Install]\nWantedBy=multi-user.target\n");
    unit
}
