#!/bin/sh
set -e

export MALLOC_ARENA_MAX=2

mkdir -p /app/storage

# 1. Pre-spawn MadelineProto IPC workers
(
    sleep 2
    echo "[Docker] Warming up IPC workers..."
    if [ -x "/app/bin/php" ]; then
        /app/bin/php /app/warmup-ipc.php || true
    elif command -v php >/dev/null 2>&1; then
        php /app/warmup-ipc.php || true
    fi
) &

# 2. Start FrankenPHP server
echo "[Docker] Starting PencariMovie Server on 0.0.0.0:${PORT:-8088}..."
if [ -f "/app/Caddyfile" ]; then
    exec /app/bin/frankenphp run --config /app/Caddyfile
else
    exec /app/bin/frankenphp php-server --listen "0.0.0.0:${PORT:-8088}" --root /app
fi
