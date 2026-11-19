#!/usr/bin/env bash
# 用 PM2 管理的 Node.js（npm）專案
# 需要先安裝 pm2 並執行過一次 `pm2 startup`，enable 才會在開機時生效
#
# syscore:name=frontend
# syscore:description=Frontend (npm + PM2)
# syscore:working_dir=/srv/frontend
# syscore:timeout=900

APP=frontend

start()   { pm2 start npm --name "$APP" -- start; }
stop()    { pm2 stop "$APP"; }
restart() { pm2 restart "$APP"; }
status()  { pm2 describe "$APP"; }
logs()    { pm2 logs "$APP" --lines "${1:-50}" --nostream; }

# 把目前的 PM2 程序清單存起來，開機時 pm2 會自動還原
enable()  { pm2 save; }

# 從 PM2 移除後再存檔，開機時就不會啟動
disable() {
    pm2 delete "$APP"
    pm2 save
}

deploy() {
    git pull --ff-only
    npm ci
    npm run build
    restart
}
