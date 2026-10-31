# syntax=docker/dockerfile:1
#
# PencariMovie Server - Alpine (musl) image.
#
#   docker build -f Dockerfile.rust -t pencarimovie-server .
#   docker run --rm -p 8088:8088 -v pmdata:/data pencarimovie-server
#
# The whole server is ONE binary: `public/` is embedded at compile time
# (`include_dir!`), so the runtime image needs nothing but the executable and a
# CA bundle. `heroku.yml` builds this same file for Heroku; the bind address is
# automatic there (the binary binds 0.0.0.0 whenever it sees a PaaS marker:
# DYNO, RENDER, K_SERVICE, or PORT without an explicit host).
#
# State (Telegram sessions, auth.json, caches, logs) lives in PM_APP_ROOT and is
# the only thing worth a volume.

FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src

# Stamped into the binary by `build.rs`. The build context deliberately has no
# `.git` and no tags, so `git describe` always fails there and the server falls
# back to `CARGO_PKG_VERSION` — which is `0.1.0`, i.e. BELOW the origin's
# `min_version` (2.7.9), so every client showed "Update available (installed
# 0.1.0)" on a build that was in fact current. Builds must pass the real
# version: `--build-arg PM_BUILD_VERSION=$(git describe --tags --always)`.
# Empty is ignored by build.rs, so a plain `docker build` still works.
ARG PM_BUILD_VERSION=
ENV PM_BUILD_VERSION=${PM_BUILD_VERSION}

# The dashboard and Linux tray script are embedded into the binary.
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY desktop ./desktop
COPY public ./public
COPY desktop ./desktop
# NO `--mount=type=cache` HERE. BuildKit cache mounts are the only way to keep the
# cargo registry and target dir between builds, and they are also the one
# instruction Heroku's CLASSIC Docker builder cannot parse at all — it stops the
# build dead with:
#
#     Step 9/18 : RUN --mount=type=cache,target=/usr/local/cargo/registry ...
#     The --mount option requires BuildKit.
#
# "Deploy to Heroku" and every one-click button use that classic builder, so
# cache mounts made the template undeployable. A plain RUN costs a full recompile
# on each source build and works everywhere. CI does not use this file: the
# release pipeline builds `Dockerfile.release` (COPY-only, from a prebuilt
# musl binary).
# Pin the ISA baseline for x86_64. 3.0.x containers died with SIGILL (exit 132)
# about a second after the banner on any CPU without AVX2 — which is every
# Synology x86 NAS (Atom C2538/C3538, Celeron J4125/J4025, N4000) and every
# pre-2013 desktop. The x86-64 baseline is already rustc's default for this
# target, so stating it explicitly cannot change a correct build; it stops a
# polluted build host (RUSTFLAGS, a stale ~/.cargo/config.toml) from silently
# raising it, which is exactly how the released artifacts broke. Never use
# `native` here. `uname -m` is the build/target arch: nothing in this file
# cross-compiles.
RUN set -eux; \
    if [ "$(uname -m)" = "x86_64" ]; then \
      export RUSTFLAGS="-C target-cpu=x86-64 -C target-feature=-avx,-avx2,-fma,-f16c"; \
    fi; \
    cargo build --release --locked \
 && cp target/release/pencarimovie-server /usr/local/bin/pencarimovie-server

FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata curl \
 && adduser -D -u 10001 -h /data pencarimovie \
 && mkdir -p /data \
 && chown -R 10001:10001 /data

COPY --from=build /usr/local/bin/pencarimovie-server /usr/local/bin/pencarimovie-server

ENV PM_APP_ROOT=/data \
    PM_LISTEN_HOST=0.0.0.0 \
    PORT=8088

EXPOSE 8088
VOLUME ["/data"]
USER 10001

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s \
  CMD curl -fsS "http://127.0.0.1:${PORT}/api/version" >/dev/null || exit 1

CMD ["pencarimovie-server"]
