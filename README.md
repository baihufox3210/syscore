# SysCore

A Discord bot for managing services on your server with plain bash scripts.

Upload a script, and every function in it becomes an action you can run from Discord: `start`, `stop`, `deploy`, `logs`, or anything else you define. It works the same way for systemd services, Docker containers, and npm apps.

## Script format

```bash
#!/usr/bin/env bash
# syscore:name=blog
# syscore:description=Blog API
# syscore:working_dir=/srv/blog
# syscore:timeout=600

SERVICE=blog-api

start()   { sudo -n systemctl start "$SERVICE"; }
stop()    { sudo -n systemctl stop "$SERVICE"; }
status()  { systemctl status "$SERVICE" --no-pager; }
logs()    { journalctl -u "$SERVICE" -n "${1:-50}" --no-pager; }

deploy() {
    git pull --ff-only
    npm ci && npm run build
    sudo -n systemctl restart "$SERVICE"
}

_helper() { :; }  # names starting with _ are not exposed
```

| Metadata      | Required | Description                                       |
| ------------- | -------- | ------------------------------------------------- |
| `name`        | no       | Task name. Defaults to the file name.             |
| `description` | no       | Shown in `/task_list`.                            |
| `working_dir` | no       | Absolute path to `cd` into before running.        |
| `timeout`     | no       | Seconds before the action is killed. Default 120, max 3600. |

Rules:

- Every top-level function is an action. Functions starting with `_` are hidden.
- The script is sourced each time an action runs, so put only variables at the top level and keep commands inside functions.
- Actions run with `set -eo pipefail`, so they stop at the first failing command.
- Arguments from Discord arrive as `$1`, `$2`, and so on. They are passed as plain values and are never run as shell code.
- stdin is closed, so `sudo` must be passwordless. See [`deploy/sudoers`](deploy/sudoers).

See [`docs/SCRIPTS.md`](docs/SCRIPTS.md) for the full rules, standard actions and patterns, and [`examples/scripts/`](examples/scripts) for a blank template and systemd, Docker Compose, and npm/PM2 examples.

Check a script before uploading it:

```sh
syscore check my-service.sh   # or: cargo run -- check my-service.sh
```

## Commands

Only the bot's application owner can use these.

| Command                              | Description                         |
| ------------------------------------ | ----------------------------------- |
| `/task_add file:<script.sh>`         | Add a task, or replace one with the same name. |
| `/task_list`                         | List tasks and their actions.       |
| `/task_info name:`                   | Show a task's settings and script.  |
| `/task_run name: action: [args:]`    | Run an action.                      |
| `/task_remove name:`                 | Delete a task and its script.       |
| `/ping`                              | Check that the bot responds.        |

## Development

```sh
cp .env.example .env   # set DISCORD_TOKEN
cargo run
cargo test
```

Invite the bot with the `bot` and `applications.commands` scopes.

## Deployment

```sh
cargo build --release

sudo mkdir -p /opt/syscore && sudo chown "$USER": /opt/syscore
install -m 755 target/release/syscore /opt/syscore/
install -m 600 .env.example /opt/syscore/.env   # then set DISCORD_TOKEN

sudo usermod -aG docker,systemd-journal "$USER"
sudo install -m 0440 deploy/sudoers /etc/sudoers.d/syscore && sudo visudo -c

# Service
sudo cp deploy/syscore.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now syscore
journalctl -u syscore -f
```

Before installing, edit `User`, `WorkingDirectory`, and `PATH` in [`deploy/syscore.service`](deploy/syscore.service), and the unit list in [`deploy/sudoers`](deploy/sudoers).

Tasks are stored in `data/` under the working directory: the registry in `tasks.json` and the scripts in `scripts/`.

## Security

Anyone who can run commands on this bot can run shell code as the bot user. Membership in the `docker` group is effectively root access. Keep the bot token secret, and grant sudo only for the exact commands your scripts need.
