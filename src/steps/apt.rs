use anyhow::Result;

use crate::session::SshSession;
use crate::ui;

pub async fn execute(session: &SshSession, packages: &[String]) -> Result<()> {
    // Check which packages are not yet installed
    let mut missing = Vec::new();
    for pkg in packages {
        let check = format!(
            "dpkg-query -W -f='${{Status}}' {} 2>/dev/null | grep -q 'install ok installed'",
            pkg
        );
        if !session.run_check(&check).await? {
            missing.push(pkg.as_str());
        }
    }

    if missing.is_empty() {
        ui::output("all packages already installed");
        return Ok(());
    }

    ui::output(&format!("installing: {}", missing.join(", ")));
    session.run("sudo apt-get update -qq").await?;
    session
        .run(&format!(
            "sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq {}",
            missing.join(" ")
        ))
        .await
}
