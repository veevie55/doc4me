# Building and releasing Doc4Me

> **Last reviewed:** 2026-11-16 · **Last updated:** 2026-11-16 · **Change:** builds are named by commit instead of a version number; the deleted x64 installer workflow is no longer described (previously: added a manual workflow to publish a selected Web App build to GitHub Pages)

## Build a chosen commit

All current workflows use only `workflow_dispatch`. Pushing code or a tag does not start a build,
deployment, or release. The platform build workflows upload artifacts and the separate Pages
workflow publishes the selected Web App to GitHub Pages. No workflow publishes a GitHub Release.

Doc4Me names its builds by the **commit** they were built from, not by a version number. `doc4me --version`, `doc4me-cli --version`, the MCP server's `serverInfo.version`
and the web user agent show the short commit (8 hex digits, `dev` for a local build without
`DOC4ME_BUILD_SHA`), and package files are named `doc4me-<commit>-<platform>-<arch>.<ext>`. The About
dialog shows neither a version number nor a commit.
Only the numeric fields that installers require keep the `[workspace.package] version` from
`Cargo.toml`: the MSI ProductVersion, the macOS `CFBundleShortVersionString`/`CFBundleVersion`,
the DEB/RPM package version, the AppStream release and the numeric Windows file version. The MSI
records the commit in its summary and in Programs and Features (`ARPCOMMENTS`), and `doc4me.exe`
carries it as its file and product version strings.

1. Open [Actions](https://github.com/veevie55/doc4me/actions) and choose a workflow.
2. Choose **Run workflow**, keeping **Use workflow from: main**. This selects the current workflow
   definition, independently of the source revision to build.
3. Set **ref** to `main` for current code, or a commit SHA (or branch) from this repository, such
   as the commit a published build came from. For macOS, choose `universal`, `aarch64`, or `x86_64` as well.
4. Start the run explicitly. Download its **Artifacts** when it finishes. They are retained for
   14 days and include `SOURCE_COMMIT.txt`, the resolved 40-character source SHA. The same SHA is
   in the run summary and is supplied to the build through `DOC4ME_BUILD_SHA`.

| Workflow | Runner / target | Artifacts |
|---|---|---|
| [Build Windows x86](../.github/workflows/build-windows-x86.yml) | Windows x64; x64-hosted x86 MSVC toolchain | 32-bit x86 MSI, portable ZIP (also runs on compatible 64-bit Windows) |
| [Build Windows x64](../.github/workflows/build-windows-x64.yml) | Windows x64; native x86_64 MSVC | 64-bit MSI, portable ZIP |
| [Build Windows ARM64](../.github/workflows/build-windows-arm64.yml) | Windows x64 runner, cross-compiled to ARM64 MSVC | MSI, portable ZIP |
| [Build macOS](../.github/workflows/build-macos.yml) | macOS; Apple Silicon, Intel, or universal | DMG, CLI ZIP |
| [Build Linux](../.github/workflows/build-linux.yml) | Ubuntu 22.04; x86_64, glibc 2.35+ | AppImage and zsync, DEB, RPM, tar.gz |
| [Build BSD (FreeBSD)](../.github/workflows/build-bsd.yml) | Native FreeBSD 14.5 x86_64 VM on a Linux runner | App/CLI tar.gz, SHA-256, binary information |
| [Build Web App](../.github/workflows/build-web.yml) | Linux; wasm32-unknown-unknown | Static site ZIP |

### GitHub Pages deployment

[Build Web App on GitHub Pages](../.github/workflows/build-web-pages.yml) builds and deploys the selected
`ref` to [https://veevie55.github.io/doc4me/](https://veevie55.github.io/doc4me/). Start it manually
from **Use workflow from: main**; the workflow only accepts deployments whose workflow definition is
running from `main`. It runs the same pinned Trunk build and validates the static output with
`packaging/web/package.sh --skip-build`, records the source SHA in `SOURCE_COMMIT.txt`, and uploads
the site artifact to the `github-pages` environment. A successful deployment replaces the current
Pages site. The ordinary [Build Web App](../.github/workflows/build-web.yml) remains an artifact-only
build and does not publish anything.

The selected revision supplies its own source, lockfile, assets, and packaging scripts; revisions
from before the switch to commit names still name their packages by version.
An older revision must support the selected target and packaging interface. Selecting an old
source revision through **ref** does not run its historical workflow definition.

The Windows x86/x64/ARM64 artifact workflows and the Windows readiness workflow also
check out the pinned [`storytold/craft-fonts`](https://github.com/storytold/craft-fonts) revision
`c1db6d1a8ed772191241c3d185e2104be84d8691`, then run
[`tools/prepare_corpus_fonts.py`](../tools/prepare_corpus_fonts.py). That helper pins Google Fonts
revision `cc39c4c2c304bc368c9b7c311ea2bcc6013299fe`, verifies the OFL Overpass, Noto Sans
Ethiopic, Roboto and Montserrat files by SHA-256. Original Source Sans Pro regular/bold/italic/bold-italic
is pinned separately to official `adobe-fonts/source-sans` revision
`ce77773581f4d454f0fa985c073bb25c721bfcf5`; its upstream license is saved as `OFL.txt`
for the package license collector. The helper adds these files to that checkout's manifest. `CRAFT_FONTS_REQUIRED=1`
requires these build inputs; package scripts copy the upstream OFL notices into the app package.
The same pinned input is used for Windows readiness so the measurement matches those package
builds. Other platform workflows continue to use their regular font inputs.

These seven platform-build jobs have read-only repository permissions and do not receive signing credentials.
Third-party actions are pinned to commits. Linux packaging tools (nFPM 2.49.0 and appimagetool
1.9.1) and Trunk 0.22.1 are pinned and their downloads checked against SHA-256 digests.
Windows ARM64 packages are unsigned. macOS packages use ad-hoc signing and are not notarized;
they do not have a trusted Developer ID signature. The BSD build means FreeBSD, not a claim
that OpenBSD, NetBSD, or every BSD architecture has been built.

The FreeBSD VM installs Rust, LLVM/libclang, CMake, Ninja, ALSA, and window-system libraries,
then builds the app and CLI natively. Its archive needs the matching system libraries on the
destination machine; it is not a statically linked universal BSD binary.

The Web App ZIP can be extracted and served over HTTP(S), or published with the Pages workflow above.
See [packaging/web/README.md](../packaging/web/README.md) for MIME types, paths, and hosting options.

## Publishing a GitHub Release

No workflow publishes a Release; the repository owner publishes Releases by hand. The workflow
that used to (`windows-x64-installer.yml`) was deleted on 2026-11-16 (commit `247aa7a`). To
prepare a build for a Release:

1. Run `cargo xtask ci`, the normally ignored interface audit
   (`cargo test -p doc4me-ui-egui i18n -- --include-ignored`), and `cargo audit`; push the source.
2. Run the platform workflows above with **ref** set to that commit, and download their Artifacts.
   Each file is named after the commit and `SOURCE_COMMIT.txt` holds its full SHA.
3. Publish the files as a Release that names the commit. The release notes are the entries under
   `[Unreleased]` in `CHANGELOG.md`, moved under a heading `## [<commit>] - YYYY-MM-DD` with the
   short commit. Headings of earlier releases keep their version numbers (history is not edited).

Doc4Me needs no version bump for a build or a Release. `cargo xtask version` remains for the
numeric installer fields above; change it only when installers must see a higher number.

Signing is optional: `packaging/windows/sign.ps1` signs the MSI when it receives either
`WINDOWS_CERTIFICATE` (base64 PFX) and `WINDOWS_CERTIFICATE_PASSWORD`, or the six Azure signing
settings (`AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`,
`AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE`). The platform workflows pass none, so their files
are unsigned.

## Readiness and local packaging

[Readiness (Windows)](../.github/workflows/readiness-windows.yml) remains a separate manual
check. It checks out current `main` and commits `docs/readiness.md` with the source SHA in its
report. It does not start after a push, tag, or another workflow. Starting a platform build does not start readiness checks.

Local packaging commands are unchanged:

```sh
# On the matching host, with that platform's build tools installed:
pwsh packaging/windows/package.ps1 -Arch x64
pwsh packaging/windows/package.ps1 -Arch x86
pwsh packaging/windows/package.ps1 -Arch arm64
packaging/macos/package.sh --arch universal
packaging/linux/package.sh
packaging/web/package.sh
```

The MSI's `UpgradeCode` upgrades earlier Doc4Me installations and does not touch WordCraft.
Bundled font licences and notices accompany the packages. Release download links in the READMEs
remain the owner's chosen upstream URLs.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-16 | minor | Builds are named by commit instead of a version number (`--version`, MCP, user agent, package file names; the About dialog shows neither); Releases are published by hand; removed the deleted x64 installer and ARM64 readiness workflows from this page. |
| 2026-11-15 | minor | Windows packages and readiness use a shared pinned OFL font set; the preparation script verifies Google Fonts hashes for Overpass and Noto Sans Ethiopic before embedding and packaging licences. |
| 2026-11-15 | minor | Added manual Windows x86 and x64 artifact workflows; both build MSI and portable ZIP from a selected ref and include source commit provenance. x86 uses the x64-hosted x86 MSVC toolchain. Neither workflow publishes a Release. |
| 2026-11-15 | minor | Added a manual GitHub Pages workflow for the Web App. It builds a selected ref with pinned Trunk, validates the static package, and deploys only from the main workflow definition to the `github-pages` environment. |
| 2026-11-12 | minor | Five independent manual build workflows with selected source refs, commit provenance and artifacts; removed automatic tag triggers from the existing workflows; documented explicit x64 releases and platform limits. |
