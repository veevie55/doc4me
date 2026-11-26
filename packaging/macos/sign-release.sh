#!/usr/bin/env bash
# Sign the universal packages from Build macOS on a separate runner. Import the Developer ID
# identity with import-cert.sh first. Use the signing scripts and DMG layout from the workflow checkout.
set -euo pipefail
# mktemp keeps the working directory private. Installed app and licence files use normal modes.
umask 022

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

[ "$#" -eq 5 ] || fail 'usage: sign-release.sh INPUT_DIR OUTPUT_DIR SOURCE_SHA VERSION FILE_NAME'
input="$(cd "$1" && pwd -P)"
output="$2"
source_sha="$3"
version="$4"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
[[ "$source_sha" =~ ^[a-f0-9]{40}$ ]] || fail 'expected a full source commit SHA'
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]] || fail 'expected a release version'
[ ! -e "$output" ] && [ ! -L "$output" ] || fail 'output directory already exists'

case "$5" in
  commit) label="${source_sha:0:8}" ;;
  version) label="$version" ;;
  version-commit) label="$version-${source_sha:0:8}" ;;
  none) label='' ;;
  *) fail 'file name must be commit, version, version-commit or none' ;;
esac
image_name="doc4me${label:+-$label}-macos-universal.dmg"
cli_name="doc4me-cli${label:+-$label}-macos-universal"
archive_name="$cli_name.zip"

check_file() {
  [ -f "$1" ] && [ ! -L "$1" ] || fail "expected a regular file: $1"
}

check_dir() {
  [ -d "$1" ] && [ ! -L "$1" ] || fail "expected a real directory: $1"
}

check_file "$input/SOURCE_COMMIT.txt"
check_file "$input/VERSION.txt"
[ "$(cat "$input/SOURCE_COMMIT.txt")" = "$source_sha" ] || fail 'source commit mismatch'
[ "$(cat "$input/VERSION.txt")" = "$version" ] || fail 'release version mismatch'

check_file "$input/$image_name"
check_file "$input/$archive_name"

[[ "${MACOS_SIGN_IDENTITY:-}" =~ ^[a-fA-F0-9]{40}$ ]] || fail 'a Developer ID Application identity is required'
check_file "${MACOS_KEYCHAIN:-}"
[ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_PASSWORD:-}" ] && [[ "${APPLE_TEAM_ID:-}" =~ ^[A-Z0-9]{10}$ ]] ||
  fail 'APPLE_ID, APPLE_PASSWORD and a valid APPLE_TEAM_ID are required for notarization'

work="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/doc4me-macos-release.XXXXXXXX")"
mount="$work/input-dmg"
mounted=0
cleanup() {
  if [ "$mounted" -eq 1 ]; then
    hdiutil detach "$mount" || hdiutil detach -force "$mount" || return 1
  fi
  rm -rf "$work"
}
trap cleanup EXIT

# Reconstruct the app layout so no symlinks, extended attributes or nested code from the image
# reach the signer. Copy the app metadata and resources that the selected build packaged.
stage="$work/dmg"
app="$stage/Doc4Me.app"
original="$mount/Doc4Me.app"
mkdir -p "$mount" "$app/Contents/MacOS" "$app/Contents/Resources"
hdiutil attach -readonly -nobrowse -noautoopen -mountpoint "$mount" "$input/$image_name"
mounted=1
for dir in "$original" "$original/Contents" "$original/Contents/MacOS" "$original/Contents/Resources"; do
  check_dir "$dir"
done
for relative in Contents/Info.plist Contents/PkgInfo Contents/MacOS/Doc4Me Contents/Resources/Doc4Me.icns; do
  check_file "$original/$relative"
  cat "$original/$relative" > "$app/$relative"
done
shopt -s nullglob
for file in "$original"/Contents/Resources/OFL-*.txt; do
  check_file "$file"
  cat "$file" > "$app/Contents/Resources/${file##*/}"
done
hdiutil detach "$mount"
mounted=0

plist="$app/Contents/Info.plist"
plutil -lint "$plist"
[ "$(plutil -extract CFBundleExecutable raw -o - "$plist")" = Doc4Me ] || fail 'unexpected app executable'
[ "$(plutil -extract Doc4MeBuildCommit raw -o - "$plist")" = "$source_sha" ] || fail 'app source commit mismatch'
[ "$(plutil -extract CFBundleShortVersionString raw -o - "$plist")" = "${version%%-*}" ] || fail 'app release version mismatch'
chmod 755 "$app/Contents/MacOS/Doc4Me"
lipo "$app/Contents/MacOS/Doc4Me" -verify_arch x86_64 arm64

# The CLI ZIP contains a single flat directory. Copy regular files by name rather than extracting
# archive paths, links or permissions into the signer filesystem.
cli_dir="$work/$cli_name"
mkdir "$cli_dir"
python3 - "$input/$archive_name" "$cli_dir" <<'PY'
from pathlib import Path
import re
import shutil
import stat
import sys
from zipfile import ZipFile

destination = Path(sys.argv[2])
prefix = f"{destination.name}/"
expected_names = {
    "doc4me-cli", "README.md", "LICENSE", "LICENSE-MIT", "LICENSE-APACHE",
    "LICENSE-UNICODE", "COPYRIGHT", "NOTICE",
}
names = set()
with ZipFile(sys.argv[1]) as archive:
    for entry in archive.infolist():
        if entry.filename == prefix and entry.is_dir():
            continue
        name = entry.filename.removeprefix(prefix)
        mode = stat.S_IFMT(entry.external_attr >> 16)
        if (
            not entry.filename.startswith(prefix)
            or (name not in expected_names and not re.fullmatch(r"OFL-[A-Za-z0-9._-]+\.txt", name))
            or mode not in (0, stat.S_IFREG)
            or name.lower() in names
        ):
            raise SystemExit(f"Unexpected file in the CLI ZIP: {entry.filename!r}")
        names.add(name.lower())
        with archive.open(entry) as source, (destination / name).open("xb") as target:
            shutil.copyfileobj(source, target)
PY
cli="$cli_dir/doc4me-cli"
check_file "$cli"
chmod 755 "$cli"
lipo "$cli" -verify_arch x86_64 arm64

sign() {
  codesign --force --sign "$MACOS_SIGN_IDENTITY" --keychain "$MACOS_KEYCHAIN" --timestamp "$@"
}

verify() {
  codesign --verify --strict --verbose=2 \
    -R "anchor apple generic and certificate leaf[subject.OU] = \"$APPLE_TEAM_ID\"" "$@"
}

notarize() {
  local file="$1" reply="$work/notary.json" id status
  local credentials=(--apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID")
  printf '==> Notarizing %s\n' "${file##*/}"
  xcrun notarytool submit "$file" "${credentials[@]}" --wait --timeout 1h --output-format json > "$reply" || true
  status="$(plutil -extract status raw -o - "$reply" 2>/dev/null || true)"
  if [ "$status" != Accepted ]; then
    id="$(plutil -extract id raw -o - "$reply" 2>/dev/null || true)"
    if [ -n "$id" ]; then
      xcrun notarytool log "$id" "${credentials[@]}" || true
    fi
    fail "notarization of ${file##*/} failed (status: ${status:-unknown})"
  fi
}

sign --options runtime --entitlements "$here/entitlements.plist" "$app/Contents/MacOS/Doc4Me"
sign --options runtime --entitlements "$here/entitlements.plist" "$app"
verify --deep "$app"
ditto -c -k --keepParent "$app" "$work/Doc4Me-notarize.zip"
notarize "$work/Doc4Me-notarize.zip"
xcrun stapler staple "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=2 "$app"

ln -s /Applications "$stage/Applications"
mkdir "$stage/.background"
cp "$here/dmg/background.tiff" "$stage/.background/background.tiff"
cp "$here/dmg/dmg-layout.DS_Store" "$stage/.DS_Store"
hdiutil makehybrid -hfs -hfs-volume-name Doc4Me -hfs-openfolder "$stage" -o "$work/raw.dmg" "$stage"
image="$work/$image_name"
hdiutil convert "$work/raw.dmg" -format UDZO -imagekey zlib-level=9 -o "$image"
sign "$image"
verify "$image"
notarize "$image"
xcrun stapler staple "$image"
xcrun stapler validate "$image"
spctl --assess --type open --context context:primary-signature --verbose=2 "$image"

sign --options runtime "$cli"
verify "$cli"
archive="$work/$archive_name"
ditto -c -k --norsrc --noextattr --keepParent "$cli_dir" "$archive"
# A standalone CLI executable cannot be stapled. The system retrieves its ticket online.
notarize "$archive"

mkdir -p "$output"
cp "$image" "$archive" "$input/SOURCE_COMMIT.txt" "$input/VERSION.txt" "$output/"
