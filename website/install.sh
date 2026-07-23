#!/bin/sh
set -eu

BIN="herdr"
CHANNEL="${HERDR_CHANNEL:-stable}"
INSTALL_DIR="${HERDR_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    echo ""
    echo "      ,ww"
    echo "     wWWWWWWW_)  herdr installer"
    echo "     \`WWWWWW'    herdr.chefgroep.nl"
    echo "      II  II"
    echo ""

    # detect platform (OnlineChefGroep fork: linux x86_64 only)
    OS="$(uname -s)"
    case "$OS" in
        Linux)  os="linux" ;;
        *)      err "unsupported OS: $OS (linux x86_64 only)" ;;
    esac

    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64)   arch="x86_64" ;;
        *)              err "unsupported architecture: $ARCH (linux x86_64 only)" ;;
    esac

    log "detected ${os}/${arch}"

    # check dependencies
    need curl
    need awk

    case "$CHANNEL" in
        stable)  MANIFEST_URL="https://herdr.chefgroep.nl/latest.json" ;;
        preview) MANIFEST_URL="https://herdr.chefgroep.nl/preview.json" ;;
        dev)     MANIFEST_URL="https://herdr.chefgroep.nl/dev.json" ;;
        *)       err "unsupported HERDR_CHANNEL: $CHANNEL (expected stable, preview, or dev)" ;;
    esac

    # Use the same manifests as Herdr's updater channels where the binary
    # supports them. The dev installer path is intentionally opt-in.
    TARGET="${os}-${arch}"
    log "fetching ${CHANNEL} release manifest..."
    MANIFEST="$(curl -fsSL --retry 3 --connect-timeout 10 --max-time 20 "$MANIFEST_URL")" \
        || err "can't reach ${MANIFEST_URL}. Please try again later; herdr.chefgroep.nl might be down. Who let the sheeps out? baaa."
    URL="$(printf '%s\n' "$MANIFEST" | awk -v target="\"${TARGET}\"" '
        /^[[:space:]]*"assets"[[:space:]]*:/ { in_assets = 1; next }
        in_assets && !in_target && /^[[:space:]]*}/ { exit }
        in_assets && index($0, target) {
            if ($0 ~ /:[[:space:]]*"/) {
                sub(/^.*:[[:space:]]*"/, "")
                sub(/".*$/, "")
                print
                exit
            }
            in_target = 1
            next
        }
        in_assets && in_target && /^[[:space:]]*"url"[[:space:]]*:/ {
            sub(/^.*:[[:space:]]*"/, "")
            sub(/".*$/, "")
            print
            exit
        }
    ')"
    VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"version"[[:space:]]*:/ { print $4; exit }')"
    BASE_VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"base_version"[[:space:]]*:/ { print $4; exit }')"
    BUILD_ID="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"build_id"[[:space:]]*:/ { print $4; exit }')"

    if [ -z "$URL" ]; then
        err "release manifest does not include a binary for ${TARGET}"
    fi

    if [ -n "$VERSION" ]; then
        log "downloading v${VERSION}..."
    elif [ -n "$BASE_VERSION" ] && [ -n "$BUILD_ID" ]; then
        log "downloading ${CHANNEL} build ${BUILD_ID} (base v${BASE_VERSION})..."
    elif [ -n "$BUILD_ID" ]; then
        log "downloading ${CHANNEL} build ${BUILD_ID}..."
    else
        log "downloading latest ${CHANNEL} release..."
    fi
    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    if ! curl -fsSL --retry 3 --connect-timeout 10 --max-time 120 "$URL" -o "${TMP}/${BIN}"; then
        err "download failed from ${URL}"
    fi

    # install
    mkdir -p "$INSTALL_DIR"
    mv "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
    chmod +x "${INSTALL_DIR}/${BIN}"

    log "installed ${BIN} to ${INSTALL_DIR}/${BIN}"

    # check PATH
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

    # verify
    if command -v "$BIN" >/dev/null 2>&1; then
        echo ""
        log "ready. run 'herdr' to get started."
    fi

    echo ""
}

log()  { printf '  \033[32m>\033[0m %s\n' "$1"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$1"; }
err()  { printf '  \033[31m✗\033[0m %s\n' "$1" >&2; exit 1; }

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        err "requires '$1' — install it first, or download a binary manually from https://herdr.dev/docs/install/"
    fi
}

main "$@"
