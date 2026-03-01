pub mod apt;
pub mod env;
pub mod file;
pub mod git;
pub mod run;
pub mod service;

use anyhow::Result;

use crate::config::{Settings, StepConfig};
use crate::session::SshSession;

pub async fn execute(step: &StepConfig, session: &SshSession, settings: &Settings) -> Result<()> {
    match step {
        StepConfig::Run {
            command,
            description: _,
            working_dir,
            screen,
            unless,
        } => {
            run::execute(
                session,
                command,
                working_dir.as_deref(),
                screen.as_deref(),
                unless.as_deref(),
            )
            .await
        }
        StepConfig::File {
            src,
            dest,
            mode,
            owner,
        } => file::execute(session, src, dest, mode.as_deref(), owner.as_deref()).await,
        StepConfig::Env { vars } => env::execute(session, vars, settings).await,
        StepConfig::Apt { packages } => apt::execute(session, packages).await,
        StepConfig::Git { repo, dest, branch } => {
            git::execute(session, repo, dest, branch.as_deref()).await
        }
        StepConfig::Service {
            name,
            command,
            working_dir,
            restart,
            env,
        } => {
            service::execute(
                session,
                name,
                command,
                working_dir.as_deref(),
                restart.as_deref(),
                env.as_ref(),
            )
            .await
        }
    }
}
