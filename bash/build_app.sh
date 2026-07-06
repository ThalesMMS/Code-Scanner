#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="$ROOT_DIR/target/release/bundle/macos/Code Scanner.app"

export PATH="/opt/homebrew/bin:/usr/local/bin:$HOME/.cargo/bin:$PATH"

cd "$ROOT_DIR"

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required to build the Tauri app." >&2
  exit 1
fi

if [[ ! -d "$ROOT_DIR/node_modules" ]]; then
  npm ci
fi

if [[ ! -d "$ROOT_DIR/ui/node_modules" ]]; then
  npm ci --prefix ui
fi

npm run tauri build -- --bundles app

if [[ ! -d "$APP_PATH" ]]; then
  echo "Build finished, but the .app bundle was not found at: $APP_PATH" >&2
  exit 1
fi

echo
echo "Built app bundle:"
echo "$APP_PATH"
