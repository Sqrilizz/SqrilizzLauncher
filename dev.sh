#!/bin/bash
set -euo pipefail

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export MODRINTH_API_URL="${MODRINTH_API_URL:-https://api.modrinth.com/v2/}"
export MODRINTH_API_URL_V3="${MODRINTH_API_URL_V3:-https://api.modrinth.com/v3/}"
export MODRINTH_LAUNCHER_META_URL="${MODRINTH_LAUNCHER_META_URL:-https://launcher-meta.modrinth.com/}"

frontend_pid=""

cleanup() {
	if [[ -n "$frontend_pid" ]] && kill -0 "$frontend_pid" 2>/dev/null; then
		kill "$frontend_pid" 2>/dev/null || true
		wait "$frontend_pid" 2>/dev/null || true
	fi
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "🚀 Starting Sqrilizz Launcher in development mode..."

	if curl -fsS --max-time 1 http://localhost:1420 >/dev/null 2>&1; then
	echo "📦 Using existing frontend on http://localhost:1420"
else
	echo "📦 Starting frontend on http://localhost:1420..."
	pnpm run --filter=@modrinth/app-frontend dev &
	frontend_pid=$!

	echo "⏳ Waiting for frontend to start..."
	frontend_ready=0
	for _ in {1..30}; do
		if curl -fsS --max-time 1 http://localhost:1420 >/dev/null 2>&1; then
			frontend_ready=1
			break
		fi
		sleep 1
	done

	if [[ "$frontend_ready" -ne 1 ]]; then
		echo "❌ Frontend failed to start on http://localhost:1420" >&2
		exit 1
	fi
fi

echo "🎮 Starting Tauri backend..."
pnpm run --filter=@modrinth/app tauri dev
