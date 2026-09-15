#!/bin/bash
# Build a drag-to-Applications installer without Finder automation.
set -euo pipefail

usage() {
  cat <<'HELP'
Usage: ./scripts/build-dmg.sh [--debug] [--target TARGET]

Builds Ginger Code.app and a compressed DMG in artifacts/.
Default: release build for this Mac's architecture.
Targets: aarch64-apple-darwin, x86_64-apple-darwin, universal-apple-darwin
--debug  Build a faster development installer.

Requires macOS, Xcode Command Line Tools, Node.js, pnpm, and Rust.
Agent CLIs must be installed separately on the destination Mac. The file editor is included.
Signing/notarization uses Tauri's signing environment variables when configured.
Without signing credentials, this produces an unsigned local development installer.
HELP
}
fail() { echo "Error: $*" >&2; exit 1; }
profile=release
target=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --debug) profile=debug; shift ;;
    --target) [[ $# -ge 2 ]] || fail '--target requires a value'; target="$2"; shift 2 ;;
    *) fail "Unknown argument: $1 (use --help)" ;;
  esac
done
[[ "$(uname -s)" == Darwin ]] || fail 'DMG builds require macOS.'
case "$target" in
  ''|aarch64-apple-darwin|x86_64-apple-darwin|universal-apple-darwin) ;;
  *) fail "Unsupported macOS target: $target" ;;
esac
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
for tool in node pnpm hdiutil xcrun; do command -v "$tool" >/dev/null || fail "Missing $tool. See README.md for setup."; done
xcrun --find clang >/dev/null 2>&1 || fail 'Install Xcode Command Line Tools: xcode-select --install'
if command -v rustup >/dev/null; then
  cargo_path="$(rustup which cargo)" || fail 'Install a Rust toolchain with rustup.'
  export PATH="$(dirname "$cargo_path"):$PATH"
fi
command -v cargo >/dev/null || fail 'Rust/Cargo is missing. Install Rust and reopen your terminal.'
if [[ -n "$target" ]] && command -v rustup >/dev/null; then
  targets=("$target")
  [[ "$target" != universal-apple-darwin ]] || targets=(aarch64-apple-darwin x86_64-apple-darwin)
  installed_targets="$(rustup target list --installed)"
  for required in "${targets[@]}"; do
    [[ "$installed_targets" == *"$required"* ]] || fail "Missing Rust target. Run: rustup target add $required"
  done
fi
pnpm install --frozen-lockfile
build_args=(build --bundles app)
[[ "$profile" != debug ]] || build_args+=(--debug)
[[ -z "$target" ]] || build_args+=(--target "$target")
pnpm tauri "${build_args[@]}"
# cargo metadata honors CARGO_TARGET_DIR and workspace Cargo configuration.
target_dir="$(cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 | node -e 'let s="";process.stdin.on("data",c=>s+=c);process.stdin.on("end",()=>process.stdout.write(JSON.parse(s).target_directory))')"
[[ -z "$target" ]] || target_dir="$target_dir/$target"
app="$target_dir/$profile/bundle/macos/Ginger Code.app"
[[ -d "$app" ]] || fail "Built app not found: $app"
version="$(node -p 'JSON.parse(require("fs").readFileSync("src-tauri/tauri.conf.json","utf8")).version')"
architecture="${target:-$(uname -m)}"
mkdir -p artifacts
output="$repo_root/artifacts/Ginger-Code-${version}-${architecture}-${profile}.dmg"
staging="$(mktemp -d "${TMPDIR:-/tmp}/ginger-dmg.XXXXXX")"
trap 'rm -rf -- "$staging"' EXIT
mkdir "$staging/image"
ditto "$app" "$staging/image/Ginger Code.app"
ln -s /Applications "$staging/image/Applications"
cat > "$staging/image/Read Me.txt" <<'INSTALL'
Drag Ginger Code.app into Applications, then open it.
Requires macOS 12 or newer. The file editor includes Vim keybindings.
Install and log in to your preferred agent CLI separately.
Open a project folder, press Command-K, and search files or actions.
An unsigned development build may be blocked by macOS Gatekeeper.
INSTALL
hdiutil create -volname 'Ginger Code' -srcfolder "$staging/image" -ov -format UDZO "$staging/installer.dmg"
# DiskImages can briefly report EAGAIN immediately after creating an image.
verified=false
for attempt in 1 2 3; do
  if hdiutil verify "$staging/installer.dmg"; then verified=true; break; fi
  [[ "$attempt" == 3 ]] || sleep "$attempt"
done
[[ "$verified" == true ]] || fail 'DMG verification failed; no installer was published.'
mv -f "$staging/installer.dmg" "$output"
printf '\nInstaller ready: %s\n' "$output"
