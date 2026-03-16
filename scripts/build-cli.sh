#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-${RUST_TARGET:-}}"
if [ -z "$TARGET" ]; then
  TARGET=$(rustc -vV | grep '^host:' | awk '{print $2}')
fi

echo "Building cc-notify CLI for target: $TARGET"

cargo build --release --manifest-path cc-notify-cli/Cargo.toml --target "$TARGET"

mkdir -p src-tauri/resources

BIN="cc-notify"
[[ "$TARGET" == *"windows"* ]] && BIN="cc-notify.exe"

cp "cc-notify-cli/target/$TARGET/release/$BIN" "src-tauri/resources/$BIN"
echo "CLI binary copied to src-tauri/resources/$BIN"
