mod config;
mod session;
mod steps;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;

use config::{OnboardConfig, substitute_step};
use session::SshSession;

#[derive(Parser)]
#[command(name = "onboard", about = "Lightweight SSH provisioning tool")]
struct Cli {
    /// Path to the TOML configuration file
    config: String,

    /// SSH host to connect to (e.g., user@host, or a Host from ~/.ssh/config)
    host: String,

    /// Dry run: print steps without executing
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load and parse config
    let content = std::fs::read_to_string(&cli.config)
        .with_context(|| format!("Failed to read config file: {}", cli.config))?;
    let mut config: OnboardConfig =
        toml::from_str(&content).with_context(|| "Failed to parse config file")?;

    // Resolve vars that reference local environment variables (values starting with $)
    for value in config.vars.values_mut() {
        if let Some(env_name) = value.strip_prefix('$') {
            *value = std::env::var(env_name)
                .with_context(|| format!("Environment variable {} is not set", env_name))?;
        }
    }

    // Apply variable substitution to all steps
    let resolved_steps: Vec<_> = config
        .steps
        .iter()
        .map(|s| substitute_step(s, &config.vars))
        .collect();

    if cli.dry_run {
        println!("Dry run: {} steps", resolved_steps.len());
        for (i, step) in resolved_steps.iter().enumerate() {
            println!("  [{}] {}", i + 1, step.description());
        }
        return Ok(());
    }

    // Connect to remote host
    println!("Connecting to {}...", cli.host);
    let session = SshSession::connect(&cli.host).await?;
    println!("Connected.\n");

    // Execute steps in order
    let total = resolved_steps.len();
    for (i, step) in resolved_steps.iter().enumerate() {
        let header = format!("[{}/{}] {}", i + 1, total, step.description());
        println!("{}", header);

        // Reset line counter before step
        ui::take_line_count();

        match steps::execute(step, &session, &config.settings).await {
            Ok(()) => {
                ui::complete_step(&header);
            }
            Err(e) => {
                // Leave output visible on error
                session.close().await?;
                return Err(e);
            }
        }
    }

    session.close().await?;
    println!("\nAll {} steps completed successfully.", total);
    Ok(())
}
