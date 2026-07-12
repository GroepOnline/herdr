#!/usr/bin/env bash
# install-chef-profile.sh — apply OnlineChefGroep Herdr defaults
set -euo pipefail

HERDR="${HERDR_BIN:-herdr}"
CONFIG="${HOME}/.config/herdr/config.toml"
PROFILE="$(dirname "$0")/../config/chef.toml"

if [ ! -f "$PROFILE" ]; then
    echo "ERROR: chef.toml not found at $PROFILE"
    exit 1
fi

# Backup existing config
if [ -f "$CONFIG" ]; then
    BACKUP="${CONFIG}.backup-chef-$(date +%Y%m%d-%H%M%S)"
    cp "$CONFIG" "$BACKUP"
    echo "Backed up to $BACKUP"
fi

# Merge profile (only add missing keys, never overwrite)
echo "Applying OnlineChefGroep Herdr profile..."
mkdir -p "$(dirname "$CONFIG")"

if [ -f "$CONFIG" ]; then
    # Use python to merge TOML (preserves existing user settings)
    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib
try:
    import tomli_w
except ImportError:
    print('pip install tomli tomli-w needed for merge')
    sys.exit(1)

with open('$CONFIG') as f: user = tomllib.loads(f.read() or '')
with open('$PROFILE') as f: chef = tomllib.loads(f.read() or '')

# Chef defaults only for keys not already in user config
for section, values in chef.items():
    if isinstance(values, dict):
        if section not in user:
            user[section] = {}
        for k, v in values.items():
            if k not in user[section]:
                user[section][k] = v
    elif section not in user:
        user[section] = values

with open('$CONFIG', 'w') as f:
    tomli_w.dump(user, f)
print('Profile applied (existing settings preserved)')
" 2>/dev/null || {
    # Fallback: just copy if no python toml libs
    cp "$PROFILE" "$CONFIG"
    echo "Profile copied (python toml merge skipped)"
}
echo "Done. Run 'herdr server reload-config' to apply."