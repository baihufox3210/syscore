#!/usr/bin/env bash
# 用 systemd 管理的服務
#
# syscore:name=blog
# syscore:description=Blog API Service
# syscore:working_dir=/srv/blog
# syscore:timeout=600

UNIT=baihu-blog-api

start()   { sudo -n systemctl start "$UNIT"; }
stop()    { sudo -n systemctl stop "$UNIT"; }
restart() { sudo -n systemctl restart "$UNIT"; }
status()  { systemctl status "$UNIT" --no-pager; }
enable()  { sudo -n systemctl enable "$UNIT"; }
disable() { sudo -n systemctl disable "$UNIT"; }
logs()    { journalctl -u "$UNIT" -n "${1:-50}" --no-pager; }

deploy() {
    git pull --ff-only
    npm ci
    npm run build
    restart
}
