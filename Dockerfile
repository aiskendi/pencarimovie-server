# syntax=docker/dockerfile:1
# Multi-arch Dockerfile leveraging pre-packaged Linux releases (from scripts/build-release.bat)
# or fallback to local files + frankenphp static binary.

FROM debian:bookworm-slim

ARG TARGETARCH

# Install runtime dependencies (ca-certificates, curl, procps)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    procps \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Check and extract pre-built tarball from dist/ if available, otherwise copy repository files
# In release workflow: dist/pencarimovie-downloader-linux-${TARGETARCH}.tar.gz
# When TARGETARCH=amd64 -> linux-x86_64; TARGETARCH=arm64 -> linux-aarch64
COPY . /tmp/repo/

RUN set -e; \
    ARCH_SUFFIX=""; \
    if [ "$TARGETARCH" = "arm64" ]; then \
        ARCH_SUFFIX="linux-aarch64"; \
    else \
        ARCH_SUFFIX="linux-x86_64"; \
    fi; \
    TAR_PATH="/tmp/repo/dist/pencarimovie-downloader-${ARCH_SUFFIX}.tar.gz"; \
    if [ -f "$TAR_PATH" ]; then \
        echo "Extracting release package: $TAR_PATH"; \
        tar -xzf "$TAR_PATH" -C /app; \
    else \
        echo "Release tarball not found at $TAR_PATH, copying workspace files directly..."; \
        cp -r /tmp/repo/public /tmp/repo/backend.php /tmp/repo/index.php /tmp/repo/router.php /tmp/repo/Caddyfile /tmp/repo/warmup-ipc.php /app/ 2>/dev/null || true; \
        if [ -d "/tmp/repo/vendor" ]; then cp -r /tmp/repo/vendor /app/; fi; \
        if [ -d "/tmp/repo/src" ]; then cp -r /tmp/repo/src /app/; fi; \
        mkdir -p /app/bin; \
        cp /tmp/repo/bin/php /app/bin/php 2>/dev/null || true; \
        cp /tmp/repo/bin/php.ini.unix /app/bin/php.ini 2>/dev/null || true; \
        if [ -f "/tmp/repo/frankenphp-${ARCH_SUFFIX}" ]; then \
            cp "/tmp/repo/frankenphp-${ARCH_SUFFIX}" /app/bin/frankenphp; \
        elif [ -f "/tmp/repo/bin/frankenphp" ]; then \
            cp /tmp/repo/bin/frankenphp /app/bin/frankenphp; \
        fi; \
    fi; \
    rm -rf /tmp/repo; \
    mkdir -p /app/storage; \
    chmod +x /app/bin/frankenphp /app/bin/php 2>/dev/null || true

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENV PATH="/app/bin:$PATH"
ENV PHP_BINDIR="/app/bin"
ENV PHPRC="/app/bin"

EXPOSE 8088

VOLUME ["/app/storage"]

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
