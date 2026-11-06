#!/usr/bin/env bash
# One local command: build the WASM front end into docs/assets/, then the site.
# Run from anywhere. The README gives the two install commands it needs: the
# wasm32-unknown-unknown target, and the wasm-bindgen CLI at the version
# Cargo.lock pins. wasm-opt is used only if present.
set -euo pipefail

cd "$(dirname "$0")"
out=../docs/assets

# Hooks live in .githooks; git reads them only once core.hooksPath points there,
# a per-clone setting nothing committed can make. Set here, first, so a build
# that fails below still leaves the clone wired. A path someone chose stays.
hooks_path=$(git -C .. config --get core.hooksPath || true)
if [ -z "$hooks_path" ]; then
  git -C .. config core.hooksPath .githooks
elif [ "$hooks_path" != ".githooks" ]; then
  echo "site/build.sh: core.hooksPath is already $hooks_path, so this repo's .githooks will not run" >&2
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "site/build.sh: wasm-bindgen is not on PATH; the README gives the command that installs it" >&2
  exit 1
fi

# The release profile lives in the workspace manifest's reach only via env here,
# so the size settings travel with this script.
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=abort

# Panic locations carry source paths into a module. Remapped, it does not publish
# where this machine keeps its toolchain, and two machines build the same bytes.
export RUSTFLAGS="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=${RUSTUP_HOME:-$HOME/.rustup}=/rustup --remap-path-prefix=$PWD=/site"

# Two modules: atelier_web on every page, atelier_graph on /graph/ only, so no
# other page downloads the graph app.
cargo build --release -p atelier-web -p atelier-graph --target wasm32-unknown-unknown

mkdir -p "$out"
for name in atelier_web atelier_graph; do
  wasm=target/wasm32-unknown-unknown/release/$name.wasm
  wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
    --out-dir "$out" --out-name "$name" "$wasm"
  if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz "$out/${name}_bg.wasm" -o "$out/${name}_bg.wasm"
  fi
done
command -v wasm-opt >/dev/null 2>&1 || echo "wasm-opt not found; skipping size pass" >&2

# Plain env, not the size profile above, for the native binary.
plain() {
  env -u CARGO_PROFILE_RELEASE_OPT_LEVEL -u CARGO_PROFILE_RELEASE_LTO \
      -u CARGO_PROFILE_RELEASE_CODEGEN_UNITS -u CARGO_PROFILE_RELEASE_PANIC -u RUSTFLAGS "$@"
}

# What built the modules, for the wasm-fresh gate. Only now: a build that failed
# above leaves the old stamp, which then disagrees with the sources.
plain ./atelier wasm-stamp write --root .. \
  --rustc "$(rustc --version)" \
  --wasm-bindgen "$(wasm-bindgen --version)" \
  --wasm-opt "$(command -v wasm-opt >/dev/null 2>&1 && wasm-opt --version || echo none)"

# The pages last, so the build fingerprints the WASM files it just wrote.
plain ./atelier build --base-path /tbaguette-skills "$@"
