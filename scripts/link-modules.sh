#!/usr/bin/env bash
# Symlink module dylibs from target directory to modules/
# Usage: ./scripts/link-modules.sh [debug|release]

set -e

PROFILE="${1:-debug}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MODULES_DIR="$PROJECT_ROOT/modules"
TARGET_DIR="$PROJECT_ROOT/target/$PROFILE"

# Determine dylib extension
case "$(uname -s)" in
    Darwin*) EXT="dylib" ;;
    Linux*)  EXT="so" ;;
    MINGW*|CYGWIN*|MSYS*) EXT="dll" ;;
    *) echo "Unknown OS"; exit 1 ;;
esac

mkdir -p "$MODULES_DIR"

echo "Linking modules from target/$PROFILE to modules/"

# Find all module dylibs and symlink them
for module in "$TARGET_DIR"/libmodule_*.${EXT}; do
    if [ -f "$module" ]; then
        name=$(basename "$module")
        ln -sf "$module" "$MODULES_DIR/$name"
        echo "  Linked: $name"
    fi
done

echo "Done!"
