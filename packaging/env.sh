# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                    workspace root
#   COMMIT                  short commit (8 hex digits), or `dev`
#   LABEL                   what names the packages: COMMIT, or VERSION with DOC4ME_FILE_NAME=version
#   VERSION                 [workspace.package] version from Cargo.toml (override: DOC4ME_VERSION);
#                           only for package metadata fields that must be numeric (deb/rpm,
#                           AppStream, Info.plist); file names use LABEL
#   DIST                    output directory for release artifacts (default: $ROOT/dist/release)
#   DOC4ME_BUILD_SHA    git commit baked into the binaries (see crates/engine/src/build_info.rs)
#   DOC4ME_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR        cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
}

VERSION="${DOC4ME_VERSION:-$(workspace_version)}"
if [ -z "$VERSION" ]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

if [ -z "${DOC4ME_BUILD_SHA:-}" ]; then
  DOC4ME_BUILD_SHA="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || true)"
fi
export DOC4ME_BUILD_SHA

# Builds are named by commit, not by version number.
case "$DOC4ME_BUILD_SHA" in
  *[!0-9a-fA-F]* | "") COMMIT=dev ;;
  *) COMMIT="$(printf '%s' "$DOC4ME_BUILD_SHA" | cut -c1-8)" ;;
esac
export COMMIT

# File names (packages, archives, the folders inside them) carry LABEL: the commit by default, or
# the version number with DOC4ME_FILE_NAME=version (the build and Release workflows' file-name
# field). A version must be a plain X.Y.Z[-pre] so it is safe in a file name.
case "${DOC4ME_FILE_NAME:-commit}" in
  commit) LABEL="$COMMIT" ;;
  version)
    if [ "${#VERSION}" -gt 64 ] ||
      ! [[ "$VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]]; then
      echo "error: DOC4ME_FILE_NAME=version needs a version like 1.2.3 or 1.2.3-rc.1, got '$VERSION'" >&2
      exit 1
    fi
    LABEL="$VERSION"
    ;;
  *)
    echo "error: DOC4ME_FILE_NAME must be commit or version, got '${DOC4ME_FILE_NAME}'" >&2
    exit 1
    ;;
esac
export LABEL
export DOC4ME_BUILD_DATE="${DOC4ME_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
}

# Copy licence and readme files that exist into a package directory.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE LICENSE-MIT LICENSE-APACHE LICENSE-UNICODE COPYRIGHT NOTICE; do
    if [ -f "$ROOT/$f" ]; then cp "$ROOT/$f" "$dest/"; fi
  done
  copy_font_licences "$dest"
}

# Every build embeds the fonts in assets/fonts (Sarabun, Inter, Source Sans 3 / Serif 4,
# JetBrains Mono, Intos / Intos Display), and Windows builds also embed the pinned craft-fonts
# selection (CRAFT_FONTS_DIR): the package carries their licences as OFL-<family>.txt.
copy_font_licences() {
  local dest="$1" f
  for f in "$ROOT"/assets/fonts/OFL-*.txt; do
    if [ -f "$f" ]; then cp "$f" "$dest/"; fi
  done
  [ -n "${CRAFT_FONTS_DIR:-}" ] || return 0
  for f in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
    if [ -f "$f" ]; then cp "$f" "$dest/OFL-$(basename "$(dirname "$f")").txt"; fi
  done
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}
