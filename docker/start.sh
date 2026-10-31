#!/bin/sh
# Container / PaaS launcher for the Rust port.
#
# Every value is an environment variable so the SAME image works on a desktop,
# Docker Compose and a PaaS:
#
#   PORT            listen port                       (default 8088; PaaS sets it)
#   PM_LISTEN_HOST  bind address                      (default 0.0.0.0 here)
#   PM_APP_ROOT     where storage/ lives              (the image sets /data)
#   PM_WORKERS      Telegram connections (one each)   (default 1)
#   PM_INFLIGHT     upload.getFile parts per conn     (default 8)
#   PM_MAX_STREAMS  concurrent /api/download bodies   (default 256)
#   PM_PAAS=1       forces the 0.0.0.0 default if PM_LISTEN_HOST is unset
#
# `exec` so the binary is PID 1 and receives SIGTERM: the server has a graceful
# shutdown path on Unix (tokio signal) and Docker's default 10 s stop timeout is
# enough for it.
set -eu

PORT="${PORT:-8088}"
HOST="${PM_LISTEN_HOST:-0.0.0.0}"

# File descriptors: the Telegram part pool plus streaming sockets exceed the
# default 1024 quickly under a TV client. Best effort - a low hard limit is not
# fatal, it just caps concurrency.
ulimit -n 65536 2>/dev/null || true

EXTRA=""
[ -n "${PM_WORKERS:-}" ]     && EXTRA="$EXTRA --workers $PM_WORKERS"
[ -n "${PM_INFLIGHT:-}" ]    && EXTRA="$EXTRA --inflight $PM_INFLIGHT"
[ -n "${PM_MAX_STREAMS:-}" ] && EXTRA="$EXTRA --max-streams $PM_MAX_STREAMS"

cd "$(dirname "$0")"
# The binary has been `pencarimovie-server` since the Rust port landed; this
# script still exec'd `./pencarimovie-rust`, a name no image ever shipped, so it
# could only ever fail with "not found". Prefer the PATH entry the images install
# (/usr/local/bin/pencarimovie-server) and fall back to a sibling copy.
BIN="pencarimovie-server"
[ -x "./$BIN" ] && BIN="./$BIN"
# shellcheck disable=SC2086
exec "$BIN" --port "$PORT" --host "$HOST" $EXTRA "$@"
