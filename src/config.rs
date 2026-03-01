use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OnboardConfig {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub steps: Vec<StepConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_shell_profile")]
    pub shell_profile: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shell_profile: default_shell_profile(),
        }
    }
}

fn default_shell_profile() -> String {
    "~/.profile".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepConfig {
    Apt {
        packages: Vec<String>,
    },
    Env {
        vars: HashMap<String, String>,
    },
    Run {
        command: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        screen: Option<String>,
        #[serde(default)]
        unless: Option<String>,
    },
    File {
        src: String,
        dest: String,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        owner: Option<String>,
    },
    Git {
        repo: String,
        dest: String,
        #[serde(default)]
        branch: Option<String>,
    },
    Service {
        name: String,
        command: String,
        #[serde(default)]
        working_dir: Option<String>,
        #[serde(default)]
        restart: Option<String>,
        #[serde(default)]
        env: Option<HashMap<String, String>>,
    },
}

impl StepConfig {
    pub fn description(&self) -> String {
        match self {
            StepConfig::Apt { packages } => {
                format!("Install apt packages: {}", packages.join(", "))
            }
            StepConfig::Env { vars } => {
                let keys: Vec<&String> = vars.keys().collect();
                format!(
                    "Set environment variables: {}",
                    keys.iter()
                        .map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            StepConfig::Run {
                description,
                command,
                screen,
                ..
            } => {
                if let Some(name) = screen {
                    description
                        .clone()
                        .unwrap_or_else(|| format!("Run in screen '{}': {}", name, command))
                } else {
                    description
                        .clone()
                        .unwrap_or_else(|| format!("Run: {}", command))
                }
            }
            StepConfig::File { src, dest, .. } => format!("Upload {} -> {}", src, dest),
            StepConfig::Git { repo, dest, .. } => format!("Clone {} -> {}", repo, dest),
            StepConfig::Service { name, .. } => format!("Configure service: {}", name),
        }
    }
}

/// Replace `{{ var_name }}` placeholders in a string with values from vars.
pub fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{ {} }}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Apply variable substitution to all string fields in a StepConfig.
pub fn substitute_step(step: &StepConfig, vars: &HashMap<String, String>) -> StepConfig {
    match step {
        StepConfig::Apt { packages } => StepConfig::Apt {
            packages: packages.iter().map(|p| substitute(p, vars)).collect(),
        },
        StepConfig::Env { vars: env_vars } => StepConfig::Env {
            vars: env_vars
                .iter()
                .map(|(k, v)| (substitute(k, vars), substitute(v, vars)))
                .collect(),
        },
        StepConfig::Run {
            command,
            description,
            working_dir,
            screen,
            unless,
        } => StepConfig::Run {
            command: substitute(command, vars),
            description: description.as_ref().map(|d| substitute(d, vars)),
            working_dir: working_dir.as_ref().map(|w| substitute(w, vars)),
            screen: screen.clone(),
            unless: unless.as_ref().map(|u| substitute(u, vars)),
        },
        StepConfig::File {
            src,
            dest,
            mode,
            owner,
        } => StepConfig::File {
            src: substitute(src, vars),
            dest: substitute(dest, vars),
            mode: mode.clone(),
            owner: owner.clone(),
        },
        StepConfig::Git { repo, dest, branch } => StepConfig::Git {
            repo: substitute(repo, vars),
            dest: substitute(dest, vars),
            branch: branch.clone(),
        },
        StepConfig::Service {
            name,
            command,
            working_dir,
            restart,
            env,
        } => StepConfig::Service {
            name: substitute(name, vars),
            command: substitute(command, vars),
            working_dir: working_dir.as_ref().map(|w| substitute(w, vars)),
            restart: restart.clone(),
            env: env.as_ref().map(|e| {
                e.iter()
                    .map(|(k, v)| (k.clone(), substitute(v, vars)))
                    .collect()
            }),
        },
    }
}
