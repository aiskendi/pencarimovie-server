#!/bin/sh
set -e

mkdir -p /app/storage

# 1. Start Addon daemon (port 8089)
if [ -x "/app/bin/addon" ]; then
    echo "[Docker] Starting compiled addon service on port 8089..."
    /app/bin/addon > /app/storage/addon.log 2>&1 &
elif [ -f "/app/addon.js" ]; then
    if command -v bun >/dev/null 2>&1; then
        echo "[Docker] Starting addon daemon with bun on port 8089..."
        bun /app/addon.js > /app/storage/addon.log 2>&1 &
    elif command -v node >/dev/null 2>&1; then
        echo "[Docker] Starting addon daemon with node on port 8089..."
        node /app/addon.js > /app/storage/addon.log 2>&1 &
    fi
fi

# 2. Pre-spawn MadelineProto IPC workers
(
    sleep 2
    echo "[Docker] Warming up IPC workers..."
    if [ -x "/app/bin/php" ]; then
        /app/bin/php /app/warmup-ipc.php || true
    elif command -v php >/dev/null 2>&1; then
        php /app/warmup-ipc.php || true
    fi
) &

# 3. Start FrankenPHP server
echo "[Docker] Starting PencariMovie Server with FrankenPHP on 0.0.0.0:8088..."
exec /app/bin/frankenphp php-server --listen 0.0.0.0:8088 --root /app
