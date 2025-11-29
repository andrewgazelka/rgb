#!/usr/bin/env bash
# Symlink plugin dylibs from target directory to plugins/
# Usage: ./scripts/link-plugins.sh [debug|release]

set -e

PROFILE="${1:-debug}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PLUGINS_DIR="$PROJECT_ROOT/plugins"
TARGET_DIR="$PROJECT_ROOT/target/$PROFILE"

# Determine dylib extension
case "$(uname -s)" in
    Darwin*) EXT="dylib" ;;
    Linux*)  EXT="so" ;;
    MINGW*|CYGWIN*|MSYS*) EXT="dll" ;;
    *) echo "Unknown OS"; exit 1 ;;
esac

mkdir -p "$PLUGINS_DIR"

echo "Linking plugins from target/$PROFILE to plugins/"

# Find all plugin dylibs and symlink them
for plugin in "$TARGET_DIR"/libplugin_*.${EXT}; do
    if [ -f "$plugin" ]; then
        name=$(basename "$plugin")
        ln -sf "$plugin" "$PLUGINS_DIR/$name"
        echo "  Linked: $name"
    fi
done

echo "Done!"
