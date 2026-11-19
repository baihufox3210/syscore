# Writing SysCore scripts

This is the reference for writing task scripts, for both people and AI agents. A script that follows these rules and passes `syscore check` will work when uploaded with `/task_add`.

## Quick start

1. Copy [`examples/scripts/template.sh`](../examples/scripts/template.sh).
2. Fill in the metadata and the actions.
3. Run `syscore check my-service.sh` and fix any `error` or `warn` lines.
4. Upload it in Discord with `/task_add file:my-service.sh`.
5. Test it with `/task_run name:my-service action:status`.

## Structure

```bash
#!/usr/bin/env bash
# syscore:name=blog                                   ← 1. metadata
# syscore:description=Blog API + nginx
# syscore:working_dir=/home/baihu/Projects/blog
# syscore:timeout=900

SERVICE=blog-api                                      ← 2. variables only
CONTAINER=blog-nginx

_wait_for_api() { ...; }                              ← 3. helpers, prefixed with _

start() { ...; }                                      ← 4. actions
stop()  { ...; }
```

### 1. Metadata

Lines in the form `# syscore:key=value`. They are comments to bash, so the `#` must stay. Do not put spaces around `=`.

| Key           | Default               | Rules                                    |
| ------------- | --------------------- | ---------------------------------------- |
| `name`        | file name without `.sh` | Letters, digits, `-` and `_` only. Uploading a script with an existing name replaces it. |
| `description` | none                  | One line, shown in `/task_list`.         |
| `working_dir` | the bot's directory   | Absolute path. Every action starts here. |
| `timeout`     | `120`                 | Seconds, from 1 to 3600. When it runs out, the action and all of its child processes are killed. |

An unknown key is an error. This catches typos such as `workdir`.

### 2. Top level

The whole script is **sourced every time any action runs**, including `status`. Only put the following at the top level:

- Variable assignments: `NAME=value`
- `set`, `export`, `readonly`, `declare`, `shopt`, `umask`
- Function definitions

Commands such as `git pull`, `cd` or `if ...` at the top level run on every action.

### 3. Helpers

Functions whose names start with `_` cannot be run from Discord. Use them for shared code.

### 4. Actions

Every top-level function that does not start with `_` is an action. Action names may contain letters, digits, `-` and `_`. A function is only detected when its definition starts at the beginning of a line:

```bash
start() { ...; }           # detected
function stop { ...; }     # detected
function restart() {       # detected
    inner() { ...; }       # not detected (indented)
}
```

## Standard actions

Use these names so that every task works the same way. Implement only the ones that make sense.

| Action    | Must do                                         | Exit code                   |
| --------- | ----------------------------------------------- | --------------------------- |
| `start`   | Start every component.                          | 0 when everything started.  |
| `stop`    | Stop every component. Stop the public-facing part first, such as nginx, then the backend. | 0 when everything stopped.  |
| `restart` | Restart, or `stop` then `start`.                | 0 on success.               |
| `status`  | Print the state of every component. Do not change anything. | 0, even when stopped: use `\|\| true` after `systemctl status`. |
| `enable`  | Start automatically on boot.                    | 0 on success.               |
| `disable` | Do not start on boot. Do not stop it now.       | 0 on success.               |
| `logs`    | Print recent logs. `$1` is the line count, default 50. | 0.                     |
| `deploy`  | Update the code, build it, and restart.         | Non-zero on any failure.    |

If a service has more than one component, for example an API and an nginx container, `start`, `stop`, `status`, `enable` and `disable` must cover all of them. Otherwise `stop` can leave the website reachable.

## Runtime environment

| Item         | Behavior                                                           |
| ------------ | ------------------------------------------------------------------ |
| Shell        | `/bin/bash` with `set -eo pipefail`. The action stops at the first failing command. |
| User         | The user running the bot. It is not root.                          |
| Directory    | `working_dir`. The script file itself is stored in `data/scripts/<name>.sh`. |
| Arguments    | The `args:` option, split on spaces, arrives as `$1`, `$2`, and so on. Values are passed as-is and are never run as shell code. |
| stdin        | Closed. Nothing can prompt for input.                              |
| TTY          | None.                                                              |
| `PATH`       | Set by `deploy/syscore.service`. Use full paths or extend `PATH` if a tool is not found. |
| Output       | stdout and stderr are both captured. Discord shows the last ~1700 characters. |
| Result       | Exit code 0 shows ✅, anything else shows ❌.                       |

## Rules

**Must:**

- Keep the `# syscore:` lines, starting with `#`.
- Keep commands inside functions. See [Top level](#2-top-level).
- Use `working_dir` to set the directory. Do not use `cd "$(dirname "${BASH_SOURCE[0]}")"`: after upload the script lives in `data/scripts/`.
- Use `sudo -n` for root commands, and allow each command in [`deploy/sudoers`](../deploy/sudoers). Without passwordless sudo the action fails.
- Make every command non-interactive. No prompts, no `-it` flags, no editors, no pagers.
- Set `timeout` when an action can take longer than 120 seconds. `deploy` usually needs 600 to 900.

**Should:**

- Pass `--no-pager` to `systemctl` and `journalctl`.
- Prefer `npm ci` over `npm install` in `deploy`.
- Use `git pull --ff-only` so that a diverged branch fails instead of creating a merge commit.
- Avoid ANSI color codes. They show up as text such as `[1m` in Discord.
- Print short progress lines, such as `▶ Building`, so a failure is easy to locate in the output.

## Patterns

### systemd service

```bash
SERVICE=blog-api

start()   { sudo -n systemctl start "$SERVICE"; }
stop()    { sudo -n systemctl stop "$SERVICE"; }
restart() { sudo -n systemctl restart "$SERVICE"; }
status()  { systemctl status "$SERVICE" --no-pager || true; }
enable()  { sudo -n systemctl enable "$SERVICE"; }
disable() { sudo -n systemctl disable "$SERVICE"; }
logs()    { journalctl -u "$SERVICE" -n "${1:-50}" --no-pager; }
```

The bot user needs the `systemd-journal` group for `logs`.

### Docker container

```bash
CONTAINER=blog-nginx

start()   { docker start "$CONTAINER"; }
stop()    { docker stop "$CONTAINER"; }
restart() { docker restart "$CONTAINER"; }
status()  { docker ps -a --filter "name=^${CONTAINER}$"; }
enable()  { docker update --restart unless-stopped "$CONTAINER"; }
disable() { docker update --restart no "$CONTAINER"; }
logs()    { docker logs --tail "${1:-50}" "$CONTAINER" 2>&1; }
```

The bot user needs the `docker` group.

### Docker Compose

See [`examples/scripts/docker-compose.sh`](../examples/scripts/docker-compose.sh). Set `working_dir` to the directory containing `compose.yaml`.

### npm with PM2

See [`examples/scripts/npm-pm2.sh`](../examples/scripts/npm-pm2.sh). Run `pm2 startup` once on the server so that `enable` takes effect on boot.

### Several components

Combine the patterns and order them:

```bash
start() {
    sudo -n systemctl start "$SERVICE"     # backend first
    docker start "$CONTAINER"
}

stop() {
    docker stop "$CONTAINER"               # public-facing part first
    sudo -n systemctl stop "$SERVICE"
}

status() {
    echo "== API";   systemctl status "$SERVICE" --no-pager || true
    echo "== nginx"; docker ps -a --filter "name=^${CONTAINER}$"
}
```

### Reusing an existing deploy script

If the project already has a deploy script, call it instead of copying it:

```bash
deploy()  { npm run deploy; }
restart() { bash scripts/deploy.sh --restart; }
```

## Checking a script

```sh
syscore check my-service.sh            # or: cargo run -- check my-service.sh
```

```
ok    my-service.sh → blog
      working_dir: /home/baihu/Projects/blog
      timeout:     900s
      actions:     start stop restart status enable disable logs deploy
warn  my-service.sh:12: sudo 建議加上 -n：...
```

- `error`: the upload will be rejected, or the script has a bash syntax error. Exit code 1.
- `warn`: the upload will work, but the action will probably fail or behave unexpectedly at runtime.

`syscore check` does not run any action. Test on the server with `status` first, then the actions that change things.

## Checklist for agents

Before handing a script over:

1. [ ] Metadata has `name`, `description` and an absolute `working_dir`, all as `# syscore:` lines.
2. [ ] The top level contains only variables, settings and functions.
3. [ ] Helpers start with `_`.
4. [ ] Every component is covered by `start`, `stop`, `status`, `enable` and `disable`.
5. [ ] `status` and `logs` do not change anything and exit 0.
6. [ ] Every `sudo` is `sudo -n`, and the exact commands are listed for the user to add to `deploy/sudoers`.
7. [ ] Nothing prompts or needs a TTY.
8. [ ] `timeout` covers the slowest action.
9. [ ] `syscore check` reports `ok` with no warnings.
