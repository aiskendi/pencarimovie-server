#!/bin/sh
set -e

export MALLOC_ARENA_MAX=2
export XDG_DATA_HOME="${XDG_DATA_HOME:-/tmp/caddy/data}"
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-/tmp/caddy/config}"

# Container memory optimization: auto-detect cgroup memory limits (Heroku, Docker, K8s, Fly.io)
# to prevent OOM/R14 and balance thread concurrency with available RAM.
export GODEBUG="${GODEBUG:-madvdontneed=1}"
export GOGC="${GOGC:-80}"

MEM_LIMIT=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null || cat /sys/fs/cgroup/memory.max 2>/dev/null || echo "")
if [ "$MEM_LIMIT" = "max" ]; then
    MEM_LIMIT=""
fi

# On 64-bit systems, unlimited memory in cgroup v1 is 9223372036854771712 or 9223372036854775807
if [ -n "$MEM_LIMIT" ] && [ "$MEM_LIMIT" -gt 0 ] 2>/dev/null && [ "$MEM_LIMIT" -le 34359738368 ]; then
    MEM_MB=$((MEM_LIMIT / 1024 / 1024))

    if [ "$MEM_MB" -le 512 ]; then
        export GOMEMLIMIT="${GOMEMLIMIT:-350MiB}"
        export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-4}"
        export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-6}"
        export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-96M}"
        export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-15s}"
        export FD_DOWNLOAD_PARALLEL_CHUNKS="${FD_DOWNLOAD_PARALLEL_CHUNKS:-2}"
    elif [ "$MEM_MB" -le 1024 ]; then
        export GOMEMLIMIT="${GOMEMLIMIT:-550MiB}"
        export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-6}"
        export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-10}"
        export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-128M}"
        export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-20s}"
        export FD_DOWNLOAD_PARALLEL_CHUNKS="${FD_DOWNLOAD_PARALLEL_CHUNKS:-2}"
    else
        TARGET_GOMEM=$((MEM_MB * 85 / 100))
        export GOMEMLIMIT="${GOMEMLIMIT:-${TARGET_GOMEM}MiB}"
        export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-12}"
        export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-20}"
        export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-256M}"
        export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-20s}"
        export FD_DOWNLOAD_PARALLEL_CHUNKS="${FD_DOWNLOAD_PARALLEL_CHUNKS:-3}"
    fi
elif [ -n "$DYNO" ]; then
    # Fallback default for Heroku dynos if cgroup read was empty
    export GOMEMLIMIT="${GOMEMLIMIT:-550MiB}"
    export FRANKENPHP_NUM_THREADS="${FRANKENPHP_NUM_THREADS:-6}"
    export FRANKENPHP_MAX_THREADS="${FRANKENPHP_MAX_THREADS:-10}"
    export PHP_MEMORY_LIMIT="${PHP_MEMORY_LIMIT:-128M}"
    export FRANKENPHP_MAX_WAIT_TIME="${FRANKENPHP_MAX_WAIT_TIME:-20s}"
    export FD_DOWNLOAD_PARALLEL_CHUNKS="${FD_DOWNLOAD_PARALLEL_CHUNKS:-2}"
fi

mkdir -p /tmp/caddy/data /tmp/caddy/config /app/storage 2>/dev/null || true
chmod 777 /app/storage 2>/dev/null || true

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
