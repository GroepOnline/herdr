#!/bin/sh
set -eu

BIN="herdr"
BASE_URL="${HERDR_BASE_URL:-https://herdr.chefgroep.nl}"
INSTALL_DIR="${HERDR_INSTALL_DIR:-$HOME/.local/bin}"
# Update channel to install: stable (default), preview, or dev. dev publishes a
# fresh build for every push to main. Override with --channel or HERDR_CHANNEL.
CHANNEL="${HERDR_CHANNEL:-stable}"

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

    # use the same manifest as `herdr update` so installs and updates agree
    # on the public release for this channel.
    TARGET="${os}-${arch}"
    log "fetching ${CHANNEL} release manifest..."
    MANIFEST="$(curl -fsSL --retry 3 --connect-timeout 10 --max-time 20 "$MANIFEST_URL")" \
        || err "can't reach ${MANIFEST_URL}. Please try again later; herdr.dev might be down. Who let the sheeps out? baaa."
    # Handles both manifest shapes: stable latest.json uses a flat
    # "target": "url" string, while preview/dev use "target": { "url": ... }.
    URL="$(printf '%s\n' "$MANIFEST" | awk -v target="\"${TARGET}\"" '
        /^[[:space:]]*"assets"[[:space:]]*:/ { in_assets = 1; next }
        in_assets && !found && /^[[:space:]]*}/ { exit }
        in_assets && !found && index($0, target) {
            rest = $0
            sub(/^[^:]*:[[:space:]]*/, "", rest)
            if (rest ~ /^"https?:/) {
                sub(/^"/, "", rest)
                sub(/".*$/, "", rest)
                print rest
                exit
            }
            found = 1
            next
        }
        found && /"url"[[:space:]]*:/ {
            sub(/^.*"url"[[:space:]]*:[[:space:]]*"/, "")
            sub(/".*$/, "")
            print
            exit
        }
    ')"
    # stable manifests expose "version"; preview/dev expose "base_version" +
    # "build_id" and are labelled base-<channel>.<build_id>.
    VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"version"[[:space:]]*:/ { print $4; exit }')"
    if [ -z "$VERSION" ]; then
        BASE_VERSION="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"base_version"[[:space:]]*:/ { print $4; exit }')"
        BUILD_ID="$(printf '%s\n' "$MANIFEST" | awk -F '"' '/^[[:space:]]*"build_id"[[:space:]]*:/ { print $4; exit }')"
        if [ -n "$BASE_VERSION" ] && [ -n "$BUILD_ID" ]; then
            VERSION="${BASE_VERSION}-${CHANNEL}.${BUILD_ID}"
        fi
    fi

    if [ -z "$URL" ]; then
        err "the ${CHANNEL} release manifest does not include a binary for ${TARGET}"
    fi

    if [ -n "$VERSION" ]; then
        log "downloading v${VERSION}..."
    else
        log "downloading latest release..."
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

    # keep future `herdr update` on the channel we just installed
    if [ "$CHANNEL" != "stable" ]; then
        if "${INSTALL_DIR}/${BIN}" channel set "$CHANNEL" >/dev/null 2>&1; then
            log "update channel set to ${CHANNEL}"
        else
            warn "run 'herdr channel set ${CHANNEL}' to keep updates on the ${CHANNEL} channel"
        fi
    fi

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
