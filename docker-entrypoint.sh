#!/bin/sh
set -e

export MALLOC_ARENA_MAX=2
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/caddy/data}"
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-/tmp/caddy/config}"

# Detect container memory quota from cgroups to prevent R14 / swap thrashing
MEM_LIMIT=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null || cat /sys/fs/cgroup/memory.max 2>/dev/null || echo 536870912)
if [ "$MEM_LIMIT" = "max" ] || [ -z "$MEM_LIMIT" ] || [ "$MEM_LIMIT" -gt 34359738368 ] 2>/dev/null; then
    MEM_LIMIT=536870912
fi
MEM_MB=$((MEM_LIMIT / 1024 / 1024))

# Auto-tune concurrency & memory limits for dyno RAM (Basic/Eco/Standard-1X: 512MB)
if [ "$MEM_MB" -le 512 ]; then
    export GOMEMLIMIT="${GOMEMLIMIT:-380MiB}"
    export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-4}"
    export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-6}"
    export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-128M}"
    export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-10s}"
elif [ "$MEM_MB" -le 1024 ]; then
    export GOMEMLIMIT="${GOMEMLIMIT:-800MiB}"
    export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-8}"
    export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-12}"
    export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-256M}"
    export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-15s}"
else
    TARGET_GOMEM=$((MEM_MB * 85 / 100))
    export GOMEMLIMIT="${GOMEMLIMIT:-${TARGET_GOMEM}MiB}"
    export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-12}"
    export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-24}"
    export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-512M}"
    export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-20s}"
fi

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
