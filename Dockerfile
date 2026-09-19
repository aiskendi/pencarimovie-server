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
    if [ "$TARGETARCH" = "arm64" ] || [ "$(uname -m)" = "aarch64" ]; then \
        ARCH_SUFFIX="linux-aarch64"; \
    else \
        ARCH_SUFFIX="linux-x86_64"; \
    fi; \
    TAR_PATH="/tmp/repo/dist/pencarimovie-downloader-${ARCH_SUFFIX}.tar.gz"; \
    mkdir -p /tmp/extract; \
    if [ -f "$TAR_PATH" ]; then \
        echo "Extracting local release package: $TAR_PATH"; \
        tar -xzf "$TAR_PATH" -C /tmp/extract; \
    elif [ -f "/tmp/repo/backend.php" ] && [ -x "/tmp/repo/bin/frankenphp" ] && [ -d "/tmp/repo/vendor" ]; then \
        echo "Copying workspace files directly..."; \
        cp -r /tmp/repo/public /tmp/repo/backend.php /tmp/repo/index.php /tmp/repo/router.php /tmp/repo/Caddyfile /tmp/repo/warmup-ipc.php /tmp/extract/ 2>/dev/null || true; \
        cp -r /tmp/repo/vendor /tmp/extract/; \
        if [ -d "/tmp/repo/src" ]; then cp -r /tmp/repo/src /tmp/extract/; fi; \
        mkdir -p /tmp/extract/bin; \
        cp /tmp/repo/bin/php /tmp/extract/bin/php 2>/dev/null || true; \
        cp /tmp/repo/bin/php.ini.unix /tmp/extract/bin/php.ini 2>/dev/null || true; \
        cp /tmp/repo/bin/frankenphp /tmp/extract/bin/frankenphp 2>/dev/null || true; \
    else \
        echo "Downloading runtime package from GitHub..."; \
        curl -fsSL -o /tmp/server.tar.gz "https://github.com/aiskendi/pencarimovie-server/releases/latest/download/pencarimovie-downloader-${ARCH_SUFFIX}.tar.gz"; \
        tar -xzf /tmp/server.tar.gz --strip-components=1 -C /tmp/extract; \
        rm -f /tmp/server.tar.gz; \
        echo "Overlaying repository files..."; \
        cp -r /tmp/repo/public /tmp/repo/backend.php /tmp/repo/index.php /tmp/repo/router.php /tmp/repo/Caddyfile /tmp/repo/warmup-ipc.php /tmp/extract/ 2>/dev/null || true; \
        if [ -d "/tmp/repo/vendor" ]; then cp -r /tmp/repo/vendor /tmp/extract/; fi; \
        if [ -d "/tmp/repo/src" ]; then cp -r /tmp/repo/src /tmp/extract/; fi; \
        if [ -f "/tmp/repo/bin/php.ini.unix" ]; then cp /tmp/repo/bin/php.ini.unix /tmp/extract/bin/php.ini 2>/dev/null || true; fi; \
    fi; \
    cp -a /tmp/extract/. /app/; \
    rm -rf /tmp/extract /tmp/repo; \
    mkdir -p /app/storage; \
    chmod -R 777 /app/storage; \
    chmod +x /app/bin/frankenphp /app/bin/php /app/bin/ffmpeg 2>/dev/null || true; \
    test -x /app/bin/frankenphp || (echo "FATAL: /app/bin/frankenphp is missing or not executable!" && exit 1)

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENV PATH="/app/bin:$PATH"
ENV PHP_BINDIR="/app/bin"
ENV PHPRC="/app/bin"

EXPOSE 8088

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["start"]
