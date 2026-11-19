#!/usr/bin/env bash
# Copy this file, fill in the values, then check it with:
#   syscore check my-service.sh
# See docs/SCRIPTS.md for the full rules.
#
# syscore:name=my-service
# syscore:description=What this service is
# syscore:working_dir=/absolute/path/to/project
# syscore:timeout=120

# Top level: variables only. This part runs before every action.
NAME=my-service

# Helpers: names start with _, not runnable from Discord.
_log() { printf '▶ %s\n' "$*"; }

# Actions: every other top-level function.
start()   { _log "Starting $NAME"; }
stop()    { _log "Stopping $NAME"; }
restart() { stop; start; }
status()  { _log "Status of $NAME"; }
enable()  { _log "Enable $NAME on boot"; }
disable() { _log "Disable $NAME on boot"; }
logs()    { _log "Last ${1:-50} log lines of $NAME"; }
deploy()  { _log "Deploying $NAME"; restart; }
