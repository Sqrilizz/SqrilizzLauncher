#!/bin/bash
# Sqrilizz Launcher Development Script
# Запускает фронтенд и бэкенд одновременно

# Workaround для WebKit segfault на Linux
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export MODRINTH_API_URL="${MODRINTH_API_URL:-https://api.modrinth.com/v2/}"
export MODRINTH_API_URL_V3="${MODRINTH_API_URL_V3:-https://api.modrinth.com/v3/}"
export MODRINTH_LAUNCHER_META_URL="${MODRINTH_LAUNCHER_META_URL:-https://launcher-meta.modrinth.com/}"

echo "🚀 Starting Sqrilizz Launcher in development mode..."

# Запуск фронтенда в фоне
echo "📦 Starting frontend on http://localhost:1420..."
pnpm run --filter=@modrinth/app-frontend dev &
FRONTEND_PID=$!

# Ждём пока фронтенд запустится
echo "⏳ Waiting for frontend to start..."
sleep 5

# Проверяем что фронтенд запустился
if ! curl -s http://localhost:1420 > /dev/null 2>&1; then
    echo "⚠️  Frontend not ready yet, waiting more..."
    sleep 5
fi

# Запуск Tauri
echo "🎮 Starting Tauri backend..."
pnpm run --filter=@modrinth/app tauri dev

# Cleanup при выходе
trap "kill $FRONTEND_PID 2>/dev/null" EXIT
