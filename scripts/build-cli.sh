#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-${RUST_TARGET:-}}"
if [ -z "$TARGET" ]; then
  TARGET=$(rustc -vV | grep '^host:' | awk '{print $2}')
fi

echo "Building local-ai-gateway CLI for target: $TARGET"

cargo build --release --manifest-path local-ai-gateway-cli/Cargo.toml --target "$TARGET"

mkdir -p src-tauri/resources

BIN="local-ai-gateway"
[[ "$TARGET" == *"windows"* ]] && BIN="local-ai-gateway.exe"

cp "local-ai-gateway-cli/target/$TARGET/release/$BIN" "src-tauri/resources/$BIN"
echo "CLI binary copied to src-tauri/resources/$BIN"
