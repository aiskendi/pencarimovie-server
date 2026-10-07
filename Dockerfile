# syntax=docker/dockerfile:1
# Multi-arch Dockerfile leveraging pre-packaged Linux releases (from scripts/build-release.sh)
# or fallback to local files + frankenphp static binary, based on lightweight Alpine Linux.

FROM alpine:latest AS builder

ARG TARGETARCH

RUN apk add --no-cache ca-certificates curl tar

WORKDIR /tmp/build

COPY . /tmp/repo/

RUN set -e; \
    ARCH_SUFFIX=""; \
    if [ "$TARGETARCH" = "arm64" ] || [ "$(uname -m)" = "aarch64" ]; then \
        ARCH_SUFFIX="linux-aarch64"; \
    else \
        ARCH_SUFFIX="linux-x86_64"; \
    fi; \
    mkdir -p /app /tmp/extract; \
    TAR_PATH="/tmp/repo/dist/pencarimovie-downloader-${ARCH_SUFFIX}.tar.gz"; \
    if [ -f "$TAR_PATH" ]; then \
        echo "Extracting local release package: $TAR_PATH"; \
        tar -xzf "$TAR_PATH" --strip-components=1 -C /tmp/extract 2>/dev/null || tar -xzf "$TAR_PATH" -C /tmp/extract; \
        if [ ! -f /tmp/extract/bin/frankenphp ]; then \
            SUBDIR=$(find /tmp/extract -mindepth 1 -maxdepth 1 -type d | head -n 1); \
            if [ -n "$SUBDIR" ] && [ -f "$SUBDIR/bin/frankenphp" ]; then \
                cp -a "$SUBDIR"/. /tmp/extract/; \
                rm -rf "$SUBDIR"; \
            fi; \
        fi; \
    elif [ -f "/tmp/repo/backend.php" ] && [ -x "/tmp/repo/bin/frankenphp" ] && [ -d "/tmp/repo/vendor" ]; then \
        echo "Copying workspace files directly..."; \
        cp -r /tmp/repo/public /tmp/repo/backend.php /tmp/repo/index.php /tmp/repo/router.php /tmp/repo/Caddyfile /tmp/extract/ 2>/dev/null || true; \
        if [ -f "/tmp/repo/.release-tag" ]; then cp /tmp/repo/.release-tag /tmp/extract/.release-tag; fi; \
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
    fi; \
    echo "Overlaying repository files..."; \
    cp -r /tmp/repo/public /tmp/repo/backend.php /tmp/repo/index.php /tmp/repo/router.php /tmp/repo/Caddyfile /tmp/extract/ 2>/dev/null || true; \
    if [ -f "/tmp/repo/.release-tag" ]; then cp /tmp/repo/.release-tag /tmp/extract/.release-tag; fi; \
    if [ -d "/tmp/repo/vendor" ]; then cp -r /tmp/repo/vendor /tmp/extract/; fi; \
    if [ -d "/tmp/repo/src" ]; then cp -r /tmp/repo/src /tmp/extract/; fi; \
    if [ -f "/tmp/repo/bin/php.ini.unix" ]; then cp /tmp/repo/bin/php.ini.unix /tmp/extract/bin/php.ini 2>/dev/null || true; fi; \
    if [ -f "/tmp/repo/bin/php" ]; then cp /tmp/repo/bin/php /tmp/extract/bin/php 2>/dev/null || true; fi; \
    ENTRY="/tmp/extract/vendor/danog/madelineproto/src/Ipc/Runner/entry.php"; \
    if [ -f "$ENTRY" ] && ! grep -q "str_ends_with(\$arguments\[0\]" "$ENTRY" 2>/dev/null; then \
        sed -i 's/\$arguments = \\array_slice(\$GLOBALS\['\''argv'\''\], 1);/\$arguments = \\array_slice(\$GLOBALS\['\''argv'\''\], 1); if (isset(\$arguments[0]) \&\& (\\str_ends_with(\$arguments[0], '\''.php'\'') || (isset(\$arguments[1]) \&\& \\in_array(\$arguments[1], ['\''madeline-ipc'\'', '\''madeline-worker'\''], true)))) { \\array_shift(\$arguments); }/g' "$ENTRY" 2>/dev/null || true; \
    fi; \
    cp -a /tmp/extract/. /app/; \
    mkdir -p /app/storage /tmp/caddy/data /tmp/caddy/config; \
    chmod -R 777 /app/storage; \
    chmod +x /app/bin/frankenphp /app/bin/php 2>/dev/null || true; \
    test -x /app/bin/frankenphp || (echo "FATAL: /app/bin/frankenphp is missing or not executable!" && exit 1)

# Final lightweight runner image
FROM alpine:latest

# Install lightweight runtime dependencies
RUN apk add --no-cache ca-certificates curl procps

WORKDIR /app

# Copy prepared application from builder stage
COPY --from=builder /app /app

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENV PATH="/app/bin:$PATH"
ENV PHP_BINDIR="/app/bin"
ENV PHPRC="/app/bin"

EXPOSE 8088

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["start"]
