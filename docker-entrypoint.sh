#!/bin/sh
set -e

export MALLOC_ARENA_MAX=2
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/caddy/data}"
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-/tmp/caddy/config}"

# On Heroku / Docker: scale FrankenPHP thread pool to use all available CPU cores
CORES=$(nproc 2>/dev/null || echo 8)
export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-$((CORES * 2))}"
export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-$((CORES * 4))}"

mkdir -p /tmp/caddy/data /tmp/caddy/config /app/storage 2>/dev/null || true
chmod 777 /app/storage 2>/dev/null || true

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

# If custom arguments were passed (and not self or 'start'), execute them
if [ $# -gt 0 ] && [ "$1" != "/usr/local/bin/docker-entrypoint.sh" ] && [ "$1" != "start" ]; then
    exec "$@"
fi

# 2. Start FrankenPHP server
PORT="${PORT:-8088}"
echo "[Docker] Starting PencariMovie Server on 0.0.0.0:${PORT}..."
if [ -f "/app/Caddyfile" ]; then
    exec /app/bin/frankenphp run --config /app/Caddyfile
else
    exec /app/bin/frankenphp php-server --listen "0.0.0.0:${PORT:-8088}" --root /app
fi
