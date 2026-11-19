#!/usr/bin/env bash
# 用 Docker Compose 管理的服務
# enable / disable 透過 restart policy 控制開機或 Docker 重啟後是否自動啟動
#
# syscore:name=wiki
# syscore:description=Wiki (Docker Compose)
# syscore:working_dir=/srv/wiki
# syscore:timeout=900

start()   { docker compose up -d; }
stop()    { docker compose stop; }
restart() { docker compose restart; }
status()  { docker compose ps; }
logs()    { docker compose logs --tail "${1:-50}" --no-color; }

enable() {
    docker compose up -d
    docker update --restart unless-stopped $(docker compose ps -q)
}

disable() {
    docker update --restart no $(docker compose ps -q)
}

deploy() {
    git pull --ff-only
    docker compose pull
    docker compose up -d --build --remove-orphans
    docker image prune -f
}
