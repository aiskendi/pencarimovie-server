#!/bin/sh
set -e

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

# 3. Start FrankenPHP server
echo "[Docker] Starting PencariMovie Server with FrankenPHP on 0.0.0.0:8088..."
exec /app/bin/frankenphp php-server --listen 0.0.0.0:8088 --root /app
