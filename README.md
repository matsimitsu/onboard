# onboard

Lightweight SSH provisioning tool. A single Rust binary that reads a TOML config and applies it to a remote host over SSH.

Think mini-Ansible — for when you just need to install a few tools, set credentials, upload configs, and start processes on a fresh VM.

## Install

```sh
cargo install --path .
```

Or download a prebuilt binary from [Releases](https://github.com/appsignal/onboard/releases).

## Usage

```sh
onboard <config.toml> <ssh-host>
```

The SSH host can be a `user@host` string or a Host from your `~/.ssh/config`.

### Dry run

Preview steps without executing:

```sh
onboard <config.toml> <ssh-host> --dry-run
```

## Config format

TOML with ordered `[[steps]]` arrays. Steps execute top-to-bottom.

```toml
[settings]
shell_profile = "~/.bashrc"   # where to append env vars (default: ~/.profile)

[vars]
api_key = "$MY_API_KEY"        # resolved from local env vars

[[steps]]
type = "apt"
packages = ["curl", "git", "build-essential"]

[[steps]]
type = "git"
repo = "git@github.com:org/repo.git"
dest = "~/app"
branch = "main"                # optional, defaults to repo default

[[steps]]
type = "run"
command = "curl https://example.com/install.sh | sh"
description = "Install toolchain"
working_dir = "~/app"          # optional
unless = "test -f /usr/local/bin/tool"  # skip if this returns 0
screen = "mytool"              # optional, runs in a detached screen session

[[steps]]
type = "env"
vars = { API_KEY = "{{ api_key }}" }

[[steps]]
type = "file"
src = "./local-config.toml"
dest = "~/.config/app/config.toml"
mode = "0644"                  # optional
owner = "root:root"            # optional

[[steps]]
type = "service"
name = "myapp"
command = "/home/user/app/start.sh"
working_dir = "/home/user/app" # optional
env = { PORT = "3000" }        # optional
restart = "on-failure"         # optional, default: on-failure
```

## Step types

| Type | Description |
|------|-------------|
| `apt` | Install packages (skips already-installed) |
| `git` | Clone or pull a repo |
| `run` | Execute a command (supports `unless`, `working_dir`, `screen`) |
| `env` | Append `export K=V` to shell profile (skips if already set) |
| `file` | Upload a local file with optional chmod/chown |
| `service` | Create a systemd unit, enable and start it |

## Variables

The `[vars]` section defines variables that can be referenced as `{{ var_name }}` in any step field.

Values starting with `$` are resolved from local environment variables:

```toml
[vars]
token = "$GITHUB_TOKEN"   # reads $GITHUB_TOKEN from your local env
```

## SSH

onboard uses your system's `ssh` binary with ControlMaster multiplexing — one TCP connection for all steps. It inherits your `~/.ssh/config` and supports SSH agent forwarding.

## Releasing

```sh
./scripts/release.sh
```

This bumps the patch version in `Cargo.toml`, commits, tags, and pushes. The release GitHub Action then builds binaries for all platforms.

## License

MIT
