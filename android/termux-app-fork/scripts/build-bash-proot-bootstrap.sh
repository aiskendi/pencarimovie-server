#!/usr/bin/env bash
# Build a minimal Termux bootstrap ZIP containing:
#   bash, proot, the termux-exec shim proot needs to link, and the
#   apt-get/dpkg stack + coreutils + curl + openssl used by
#   NativeRunner.tryStartNativePhp() to install PHP on demand.
#
# Source: the existing full bootstrap-aarch64.zip (137 MB).
# Output: bootstrap-aarch64.zip with the same layout + SYMLINKS.txt.
#
# Usage:
#   ./scripts/build-bash-proot-bootstrap.sh <source.zip> <output.zip>
#
# The dependency closure is resolved with readelf -d (DT_NEEDED) and followed
# transitively. Runs in seconds on Linux.
#
# NOTE: `pkg` is deliberately NOT shipped. The bundled pkg script is the stock
# Termux one and hardcodes /data/data/com.termux/... paths, so it cannot work
# under com.pencarimovie.downloader. tryStartNativePhp() uses the apt-get
# branch (apt-get update / install -d + dpkg-deb -x), which needs no pkg.

set -euo pipefail

SRC_ZIP="${1:?usage: build-bash-proot-bootstrap.sh <source.zip> <output.zip>}"
OUT_ZIP="${2:?usage: build-bash-proot-bootstrap.sh <source.zip> <output.zip>}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

[ -f "$SRC_ZIP" ] || { echo "Source bootstrap not found: $SRC_ZIP" >&2; exit 1; }

SRC_SIZE=$(stat -c%s "$SRC_ZIP")
echo "Source: $SRC_ZIP ($(( SRC_SIZE / 1048576 )) MB)"

# --- Extract bin/, lib/, libexec/, etc/apt/, SYMLINKS.txt -------------------
# libexec/proot/loader is REQUIRED: proot is built with PROOT_UNBUNDLE_LOADER,
# so it execs its own statically-linked loader instead of the target binary.
# Without the loader, proot falls back to a direct execve() of the target,
# which needs the ELF interpreter /system/bin/linker64 -> /apex/... and fails
# with ENOENT because /apex is not traversable by untrusted apps on Android 10+.
mkdir -p "$WORK/src"
unzip -q -o "$SRC_ZIP" 'bin/*' 'lib/*' 'libexec/*' 'etc/apt/*' 'etc/tls/*' \
    'share/termux-keyring/*' 'SYMLINKS.txt' -d "$WORK/src"
SRC="$WORK/src"

# --- What the runtime actually executes -------------------------------------
# bash + proot (server launch) and the apt/dpkg stack + coreutils + curl +
# openssl (on-demand PHP install in tryStartNativePhp).
#
# termux-exec: proot needs libtermux-exec.so, a symlink to
# libtermux-exec_nos_c_tre.so. The helper scripts below are only needed to
# *build* that symlink; we create it directly in SYMLINKS.txt.
BINARIES=(
    bash proot
    termux-exec-system-linker-exec
    termux-exec-ld-preload-lib
    # coreutils multi-call binary — provides find/cp/mkdir/rm/chmod/basename/
    # head/tr/mv/printf/echo/test/true/false/seq/tee/which/kill/date/nohup/
    # timeout/mktemp/readlink/stat/du/df/env/id/uname/sleep/... via symlinks.
    coreutils
    # apt/dpkg stack used by tryStartNativePhp()'s apt-get branch.
    apt apt-get apt-cache apt-config apt-mark apt-key
    dpkg dpkg-deb dpkg-query dpkg-divert dpkg-split dpkg-trigger
    dpkg-realpath dpkg-scanpackages
    gpgv
    # TLS + HTTP transport for apt.
    curl openssl
    # find is a standalone binary in Termux (not a coreutils symlink) and is
    # used by tryStartNativePhp() to locate the downloaded .deb files.
    find
    # dpkg --configure -a requires start-stop-daemon on PATH.
    start-stop-daemon
    # GNU tar. dpkg-deb shells out to `tar` with GNU-only options
    # (--warning=no-timestamp). Android's toybox /system/bin/tar rejects them:
    #   tar: Unknown option 'warning=no-timestamp'
    #   dpkg-deb: error: tar subprocess returned error exit status 1
    # Shipping GNU tar in $PREFIX/bin puts it ahead of /system/bin on PATH.
    tar
)

# Multi-call coreutils symlink names the runtime relies on.
COREUTILS_LINKS=(
    ls cp mv rm mkdir rmdir cat head tail tr cut sort uniq wc touch chmod chown
    ln readlink realpath stat du df id uname sleep env printf echo expr test
    true false seq tee basename dirname which kill date od xxd mktemp install
    sync nohup timeout yes whoami groups pwd link unlink shred split csplit fold
    fmt pr join paste comm tsort expand unexpand nl numfmt truncate shuf base32
    base64 basenc cksum sum md5sum sha1sum sha224sum sha256sum sha384sum
    sha512sum b2sum dd stty tty logname hostid arch nproc chroot runcon stdbuf
    pathchk factor users who uptime nice mkfifo mknod chcon chgrp dir vdir
    dircolors
)

# --- Resolve the lib closure ------------------------------------------------
# Map soname -> real file from SYMLINKS.txt (e.g. libreadline.so.8 -> libreadline.so.8.3).
declare -A SONAME_TO_REAL
while IFS= read -r line; do
    [[ "$line" == *"←"* ]] || continue
    target="${line%%←*}"
    link="${line##*←}"
    if [[ "$link" =~ ^\./lib/(.+)$ ]]; then
        soname="${BASH_REMATCH[1]}"
        real="${target##*/}"
        [ -f "$SRC/lib/$real" ] && SONAME_TO_REAL["$soname"]="$real"
    fi
done < "$SRC/SYMLINKS.txt"
echo "Soname map: ${#SONAME_TO_REAL[@]} entries"

declare -A NEEDED
queue=()

enqueue() { NEEDED["$1"]=0; queue+=("$1"); }

for b in "${BINARIES[@]}"; do
    [ -f "$SRC/bin/$b" ] || { echo "warning: missing bin/$b" >&2; continue; }
    while IFS= read -r lib; do
        [ -n "$lib" ] && [ -z "${NEEDED[$lib]+x}" ] && enqueue "$lib"
    done < <(readelf -d "$SRC/bin/$b" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p')
done

# apt method drivers are exec'd by apt, not by us, so they are not in BINARIES.
# They still need their own DT_NEEDED closure resolved — notably
# lib/apt/methods/http links libgnutls.so for HTTPS support.
for m in "$SRC/lib/apt/methods/"* "$SRC/lib/apt/apt-helper"; do
    [ -f "$m" ] || continue
    while IFS= read -r lib; do
        [ -n "$lib" ] && [ -z "${NEEDED[$lib]+x}" ] && enqueue "$lib"
    done < <(readelf -d "$m" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p')
done

while [ ${#queue[@]} -gt 0 ]; do
    lib="${queue[0]}"; queue=("${queue[@]:1}")
    [ "${NEEDED[$lib]}" = "1" ] && continue

    real="$lib"
    [ -f "$SRC/lib/$real" ] || real="${SONAME_TO_REAL[$lib]:-}"
    [ -n "$real" ] && [ -f "$SRC/lib/$real" ] || continue   # bionic libc/libm/libdl/liblog

    NEEDED["$real"]=1
    while IFS= read -r dep; do
        [ -n "$dep" ] && [ -z "${NEEDED[$dep]+x}" ] && enqueue "$dep"
    done < <(readelf -d "$SRC/lib/$real" 2>/dev/null | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p')
done

LIBS=()
for k in "${!NEEDED[@]}"; do [ "${NEEDED[$k]}" = "1" ] && LIBS+=("$k"); done
echo "Resolved ${#LIBS[@]} shared libraries"

# --- Build the output ZIP ---------------------------------------------------
STAGE="$WORK/stage"
mkdir -p "$STAGE/bin" "$STAGE/lib" "$STAGE/libexec/proot" "$STAGE/tmp" "$STAGE/etc" "$STAGE/home" "$STAGE/root"

for b in "${BINARIES[@]}"; do
    [ -f "$SRC/bin/$b" ] && cp "$SRC/bin/$b" "$STAGE/bin/$b"
done
for l in "${LIBS[@]}"; do
    cp "$SRC/lib/$l" "$STAGE/lib/$l"
done

# proot's unbundled loader — without this proot cannot exec anything.
for ld in loader loader32; do
    if [ -f "$SRC/libexec/proot/$ld" ]; then
        cp "$SRC/libexec/proot/$ld" "$STAGE/libexec/proot/$ld"
        chmod 755 "$STAGE/libexec/proot/$ld"
    else
        echo "ERROR: libexec/proot/$ld missing from source bootstrap" >&2
        exit 1
    fi
done

# apt config (sources.list) — apt-get update needs it.
if [ -d "$SRC/etc/apt" ]; then
    mkdir -p "$STAGE/etc/apt"
    cp -R "$SRC/etc/apt/." "$STAGE/etc/apt/"
fi

# CA bundle — apt's https method verifies the server certificate against
# etc/tls/cert.pem. Without it:
#   Certificate verification failed: The certificate is NOT trusted.
#   W: ... No system certificates available. Try installing ca-certificates.
if [ -d "$SRC/etc/tls" ]; then
    mkdir -p "$STAGE/etc/tls"
    cp -R "$SRC/etc/tls/." "$STAGE/etc/tls/"
else
    echo "ERROR: etc/tls missing from source bootstrap (CA bundle)" >&2
    exit 1
fi

# Termux repo signing keys. The termux-keyring package installs these into
# etc/apt/trusted.gpg.d/; the source files live in share/termux-keyring/.
# Without them apt rejects the repo:
#   NO_PUBKEY 5A897D96E57CF20C
#   E: The repository ... is not signed.
if [ -d "$SRC/share/termux-keyring" ]; then
    mkdir -p "$STAGE/etc/apt/trusted.gpg.d"
    cp "$SRC/share/termux-keyring/"*.gpg "$STAGE/etc/apt/trusted.gpg.d/" 2>/dev/null || true
    chmod 644 "$STAGE/etc/apt/trusted.gpg.d/"*.gpg 2>/dev/null || true
else
    echo "ERROR: share/termux-keyring missing from source bootstrap (repo keys)" >&2
    exit 1
fi

# apt method drivers — apt execs lib/apt/methods/<scheme> for each transport.
# Without these, `apt-get update` fails with:
#   E: The method driver .../lib/apt/methods/https could not be found.
# lib/apt/apt-helper is also required by several apt operations.
if [ -d "$SRC/lib/apt" ]; then
    mkdir -p "$STAGE/lib/apt"
    cp -R "$SRC/lib/apt/." "$STAGE/lib/apt/"
    chmod 755 "$STAGE/lib/apt/apt-helper" 2>/dev/null || true
    chmod 755 "$STAGE/lib/apt/methods/"* 2>/dev/null || true
else
    echo "ERROR: lib/apt missing from source bootstrap (apt method drivers)" >&2
    exit 1
fi

# dpkg helper scripts (dpkg-db-backup / dpkg-db-keeper) live in libexec/dpkg.
if [ -d "$SRC/libexec/dpkg" ]; then
    mkdir -p "$STAGE/libexec/dpkg"
    cp -R "$SRC/libexec/dpkg/." "$STAGE/libexec/dpkg/"
fi

# --- SYMLINKS.txt -----------------------------------------------------------
# TermuxInstaller reads this and calls Os.symlink(oldPath, newPath).
# Format: <target><U+2190><linkpath>
{
    for n in "${COREUTILS_LINKS[@]}"; do
        printf 'coreutils←./bin/%s\n' "$n"
    done
    # Termux ships /bin/sh as a symlink to bash. Keep it so `sh -c` works.
    printf 'bash←./bin/sh\n'
    # apt looks up lib/apt/methods/<scheme> for each URL scheme. Termux's
    # `http` method is built with GnuTLS and handles HTTPS too, but there is no
    # separate `https` binary — so point https at http. Without this,
    # `apt-get update` fails with:
    #   E: The method driver .../lib/apt/methods/https could not be found.
    printf 'http←./lib/apt/methods/https\n'
    for soname in "${!SONAME_TO_REAL[@]}"; do
        real="${SONAME_TO_REAL[$soname]}"
        if [ "${NEEDED[$real]:-}" = "1" ]; then
            printf '%s←./lib/%s\n' "$real" "$soname"
        fi
    done
    if [ "${NEEDED[libtermux-exec_nos_c_tre.so]:-}" = "1" ]; then
        printf 'libtermux-exec_nos_c_tre.so←./lib/libtermux-exec.so\n'
    fi
} | sort -u > "$STAGE/SYMLINKS.txt"

# --- Zip --------------------------------------------------------------------
OUT_ABS="$(cd "$(dirname "$OUT_ZIP")" && pwd)/$(basename "$OUT_ZIP")"
rm -f "$OUT_ABS"
( cd "$STAGE" && zip -q -r -9 "$OUT_ABS" . )

OUT_SIZE=$(stat -c%s "$OUT_ZIP")
echo ""
echo "Wrote $OUT_ZIP"
echo "  size: $(( OUT_SIZE / 1048576 )) MB (was $(( SRC_SIZE / 1048576 )) MB)"
echo "  bins: ${#BINARIES[@]}  libs: ${#LIBS[@]}  symlinks: $(wc -l < "$STAGE/SYMLINKS.txt")"
echo ""
echo "Contents:"
( cd "$STAGE" && ls -la bin/ && echo "--- libs ---" && ls lib/ && echo "--- libexec/proot ---" && ls -la libexec/proot/ )

# --- Sanity checks ----------------------------------------------------------
echo ""
echo "=== sanity checks ==="
fail=0
# Real files that must be in the ZIP.
for f in bin/bash bin/proot libexec/proot/loader libexec/proot/loader32 \
         bin/coreutils bin/apt-get bin/dpkg bin/dpkg-deb bin/curl bin/openssl \
         bin/find bin/start-stop-daemon bin/tar \
         lib/apt/apt-helper lib/apt/methods/http lib/apt/methods/gpgv \
         etc/tls/cert.pem etc/apt/sources.list \
         etc/apt/trusted.gpg.d/termux-autobuilds.gpg; do
    if [ -f "$STAGE/$f" ]; then
        echo "OK   $f"
    else
        echo "FAIL $f MISSING"
        fail=1
    fi
done

# coreutils multi-call names are symlinks created at install time from
# SYMLINKS.txt, so they are legitimately absent from the ZIP.
for n in cp mkdir rm chmod basename head tr mv printf echo test; do
    if grep -q "^coreutils←\./bin/$n$" "$STAGE/SYMLINKS.txt"; then
        echo "OK   bin/$n -> coreutils (symlink)"
    else
        echo "FAIL bin/$n not declared in SYMLINKS.txt"
        fail=1
    fi
done
# proot must reference the app's own libexec path, not com.termux.
# Note: capture strings to a var first — `strings | grep -q` trips SIGPIPE
# under `set -o pipefail` and reports a false failure.
PROOT_STRINGS="$(strings "$STAGE/bin/proot" || true)"
case "$PROOT_STRINGS" in
    *'/data/data/com.pencarimovie.downloader/files/usr/libexec/proot/loader'*)
        echo "OK   proot loader path is com.pencarimovie.downloader" ;;
    *)
        echo "FAIL proot loader path is not com.pencarimovie.downloader"
        fail=1 ;;
esac
case "$PROOT_STRINGS" in
    *'/data/data/com.termux/files/usr/libexec/proot/loader'*)
        echo "FAIL proot still points at com.termux libexec"
        fail=1 ;;
esac
[ "$fail" -eq 0 ] || { echo "SANITY CHECKS FAILED" >&2; exit 1; }
echo "all sanity checks passed"
