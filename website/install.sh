#!/bin/sh
set -eu

BIN="herdr"
BASE_URL="${HERDR_BASE_URL:-https://herdr.chefgroep.nl}"
INSTALL_DIR="${HERDR_INSTALL_DIR:-$HOME/.local/bin}"
# Update channel to install: stable (default), preview, or dev. dev publishes a
# fresh build for every push to main. Override with --channel or HERDR_CHANNEL.
CHANNEL="${HERDR_CHANNEL:-stable}"
TMP=""

cleanup() {
    if [ -n "$TMP" ]; then
        rm -f "$TMP"
    fi
}
trap cleanup EXIT HUP INT TERM

main() {
    # parse options (e.g. `curl ... | sh -s -- --channel dev`)
    while [ $# -gt 0 ]; do
        case "$1" in
            --channel|-c)
                shift
                [ $# -gt 0 ] || err "--channel requires a value (stable, preview, or dev)"
                CHANNEL="$1"
                ;;
            --channel=*) CHANNEL="${1#*=}" ;;
            -h|--help)
                echo "usage: install.sh [--channel <stable|preview|dev>]"
                exit 0
                ;;
            *) err "unknown option: $1 (use --channel <stable|preview|dev>)" ;;
        esac
        shift
    done

    case "$CHANNEL" in
        stable)  MANIFEST_FILE="latest.json" ;;
        preview) MANIFEST_FILE="preview.json" ;;
        dev)     MANIFEST_FILE="dev.json" ;;
        *) err "unknown channel: ${CHANNEL} (use stable, preview, or dev)" ;;
    esac
    MANIFEST_URL="${BASE_URL}/${MANIFEST_FILE}"

    echo ""
    echo "      ,ww"
    echo "     wWWWWWWW_)  herdr installer"
    echo "     \`WWWWWW'    herdr.chefgroep.nl"
    echo "      II  II"
    echo ""

    OS="$(uname -s)"
    case "$OS" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        *)      err "unsupported OS: $OS (supported: Linux and macOS)" ;;
    esac

    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "unsupported architecture: $ARCH (supported: x86_64 and aarch64/arm64)" ;;
    esac

    log "detected ${os}/${arch}"

    need curl
    need awk
    need tr
    # sha256_file always runs in a command substitution, so its own failure
    # cannot stop the script. Fail here, before anything is downloaded.
    if ! command -v sha256sum >/dev/null 2>&1 \
        && ! command -v shasum >/dev/null 2>&1 \
        && ! command -v openssl >/dev/null 2>&1; then
        err "requires 'sha256sum', 'shasum', or 'openssl' to verify the downloaded binary"
    fi

    TARGET="${os}-${arch}"
    log "fetching ${CHANNEL} release manifest..."
    MANIFEST="$(curl -fsSL --retry 3 --connect-timeout 10 --max-time 20 "$MANIFEST_URL")" \
        || err "can't reach ${MANIFEST_URL}. Please try again later; herdr.chefgroep.nl might be down."

    # Every installable manifest entry is a checksummed object:
    #   "target": { "url": "...", "sha256": "..." }
    URL="$(manifest_asset_field "$MANIFEST" "$TARGET" "url")"
    EXPECTED_SHA256="$(manifest_asset_field "$MANIFEST" "$TARGET" "sha256" | tr 'A-F' 'a-f')"

    VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"version"[[:space:]]*:/ { print $4; exit }')"
    if [ -z "$VERSION" ]; then
        BASE_VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"base_version"[[:space:]]*:/ { print $4; exit }')"
        BUILD_ID="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"build_id"[[:space:]]*:/ { print $4; exit }')"
        if [ -n "$BASE_VERSION" ] && [ -n "$BUILD_ID" ]; then
            VERSION="${BASE_VERSION}-${CHANNEL}.${BUILD_ID}"
        fi
    fi

    [ -n "$URL" ] || err "the ${CHANNEL} release manifest does not include a binary URL for ${TARGET}"
    case "$EXPECTED_SHA256" in
        *[!0-9a-f]*|'') err "the ${CHANNEL} release manifest has an invalid SHA-256 for ${TARGET}" ;;
    esac
    [ "${#EXPECTED_SHA256}" -eq 64 ] \
        || err "the ${CHANNEL} release manifest has an invalid SHA-256 for ${TARGET}"

    if [ -n "$VERSION" ]; then
        log "downloading v${VERSION}..."
    else
        log "downloading latest release..."
    fi

    # Create the temporary file in the destination directory. Verification
    # happens before chmod/rename, so a failed or interrupted install never
    # replaces an existing binary and the final move stays on one filesystem.
    mkdir -p "$INSTALL_DIR"
    TMP="$(mktemp "${INSTALL_DIR}/.herdr-install.XXXXXX")"

    if ! curl -fsSL --retry 3 --connect-timeout 10 --max-time 120 "$URL" -o "$TMP"; then
        err "download failed from ${URL}"
    fi

    ACTUAL_SHA256="$(sha256_file "$TMP")"
    if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
        err "checksum mismatch for ${TARGET}: expected ${EXPECTED_SHA256}, got ${ACTUAL_SHA256}"
    fi

    chmod +x "$TMP"
    mv -f "$TMP" "${INSTALL_DIR}/${BIN}"
    TMP=""

    log "verified SHA-256 and installed ${BIN} to ${INSTALL_DIR}/${BIN}"

    if [ "$CHANNEL" != "stable" ]; then
        if "${INSTALL_DIR}/${BIN}" channel set "$CHANNEL" >/dev/null 2>&1; then
            log "update channel set to ${CHANNEL}"
        else
            warn "run 'herdr channel set ${CHANNEL}' to keep updates on the ${CHANNEL} channel"
        fi
    fi

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            warn "${INSTALL_DIR} is not in your PATH"
            echo "  add it to your shell config:"
            echo ""
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            ;;
    esac

    if command -v "$BIN" >/dev/null 2>&1; then
        echo ""
        log "ready. run 'herdr' to get started."
    fi

    echo ""
}

manifest_asset_field() {
    manifest="$1"
    target="$2"
    field="$3"
    printf '%s\n' "$manifest" | awk \
        -v target="\"${target}\"" \
        -v field="\"${field}\"" '
        /^[[:space:]]*"assets"[[:space:]]*:/ { in_assets = 1; next }
        in_assets && !in_target && index($0, target) { in_target = 1; next }
        in_target && index($0, field) {
            value = $0
            sub(/^.*:[[:space:]]*"/, "", value)
            sub(/".*$/, "", value)
            print value
            exit
        }
        in_target && /^[[:space:]]*}/ { exit }
    '
}

sha256_file() {
    path="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$path" | awk '{ print tolower($1) }'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$path" | awk '{ print tolower($1) }'
    else
        openssl dgst -sha256 "$path" | awk '{ print tolower($NF) }'
    fi
}

log()  { printf '  \033[32m>\033[0m %s\n' "$1"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$1"; }
err()  { printf '  \033[31m✗\033[0m %s\n' "$1" >&2; exit 1; }

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        err "requires '$1' — install it first, or download a binary manually from https://herdr.chefgroep.nl/docs/install/"
    fi
}

main "$@"
