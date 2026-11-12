# Building and releasing Doc4Me

> **Last reviewed:** 2026-11-12 · **Last updated:** 2026-11-12 · **Change:** minor (independent manual builds, selected source refs, and no automatic push/tag triggers)

## Build a chosen commit

All current workflows use only `workflow_dispatch`. Every job also checks that the event is
`workflow_dispatch`. Pushing code or a tag does not start these workflows. The build workflows
upload artifacts; they do not deploy the Web App or publish GitHub Releases.

1. Open [Actions](https://github.com/veevie55/doc4me/actions) and choose a workflow.
2. Choose **Run workflow**, keeping **Use workflow from: main**. This selects the current workflow
   definition, independently of the source revision to build.
3. Set **ref** to `main` for current code, `v1.1.3` for the released source, or a branch, tag, or
   commit SHA from this repository. For macOS, choose `universal`, `aarch64`, or `x86_64` as well.
4. Start the run explicitly. Download its **Artifacts** when it finishes. They are retained for
   14 days and include `SOURCE_COMMIT.txt`, the resolved 40-character source SHA. The same SHA is
   in the run summary and is supplied to the build through `DOC4ME_BUILD_SHA`.

| Workflow | Runner / target | Artifacts |
|---|---|---|
| [Build Windows ARM64](../.github/workflows/build-windows-arm64.yml) | Windows x64 runner, cross-compiled to ARM64 MSVC | MSI, portable ZIP |
| [Build macOS](../.github/workflows/build-macos.yml) | macOS; Apple Silicon, Intel, or universal | DMG, CLI ZIP |
| [Build Linux](../.github/workflows/build-linux.yml) | Ubuntu 22.04; x86_64, glibc 2.35+ | AppImage and zsync, DEB, RPM, tar.gz |
| [Build BSD (FreeBSD)](../.github/workflows/build-bsd.yml) | Native FreeBSD 14.5 x86_64 VM on a Linux runner | App/CLI tar.gz, SHA-256, binary information |
| [Build Web App](../.github/workflows/build-web.yml) | Linux; wasm32-unknown-unknown | Static site ZIP |

The selected revision supplies its own source, lockfile, assets, version, and packaging scripts.
An older revision must support the selected target and packaging interface. Selecting an old
source revision through **ref** does not run its historical workflow definition.

These five jobs have read-only repository permissions and do not receive signing credentials.
Third-party actions are pinned to commits. Linux packaging tools (nFPM 2.49.0 and appimagetool
1.9.1) and Trunk 0.22.1 are pinned and their downloads checked against SHA-256 digests.
Windows ARM64 packages are unsigned. macOS packages use ad-hoc signing and are not notarized;
they do not have a trusted Developer ID signature. The BSD build means FreeBSD, not a claim
that OpenBSD, NetBSD, or every BSD architecture has been built.

The FreeBSD VM installs Rust, LLVM/libclang, CMake, Ninja, ALSA, and window-system libraries,
then builds the app and CLI natively. Its archive needs the matching system libraries on the
destination machine; it is not a statically linked universal BSD binary.

The Web App ZIP must be extracted and served over HTTP(S). See
[packaging/web/README.md](../packaging/web/README.md) for MIME types, paths, and hosting options.
The workflow does not publish to GitHub Pages or another hosting service.

## Windows x64 and explicit GitHub Releases

The existing [Windows x64 installer](../.github/workflows/windows-x64-installer.yml) is also
manual-only. With **release** left off, it builds MSI and portable ZIP artifacts. Its optional
**commit** input accepts a full 40-character SHA that is an ancestor of the run's `main` commit;
leave it empty to build the run's current source. Unlike the five workflows above, this field
is named **commit** and accepts a full main-history SHA, not a branch name.

Publishing a new version is a separate, explicit decision:

1. Only when a release is requested, set the version with `cargo xtask version set x.y.z` and
   move the relevant `CHANGELOG.md` entries from `[Unreleased]` under `## [x.y.z] - YYYY-MM-DD`.
2. Run `cargo xtask ci`, the normally ignored interface audit
   (`cargo test -p doc4me-ui-egui i18n -- --include-ignored`), and `cargo audit`; push the prepared source.
3. Choose **Windows x64 installer → Run workflow** on `main`, explicitly enabling **release**.
   That job creates the version tag and release, or stops if the tag already exists. A tag push
   alone never starts the current workflow. Ordinary builds do not require a version bump.

For an earlier version, provide **commit** and explicitly enable **release**. The workflow checks
that the existing version tag points to that commit and publishes without marking it as the
latest release. GitHub can reject tag creation at an older commit whose workflow files differ
from the default branch. If its tag must be created from a clone, create it first, then start
**Run workflow** explicitly; creating the tag is not a build or release request.

Do not change `main`, especially its workflows, during a requested release run. GitHub can
reject a release target whose workflow files change relative to the default branch. Release
notes come from the version's `CHANGELOG.md` section. Notes over GitHub's size limit retain
the section headings and each entry's first line, linking to the complete changelog.

Signing remains optional in this x64 workflow. Builds run without signing secrets, then the
packaging/signing step can receive either `WINDOWS_CERTIFICATE` (base64 PFX) and
`WINDOWS_CERTIFICATE_PASSWORD`, or the six Azure signing settings (`AZURE_TENANT_ID`,
`AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`,
`AZURE_CERT_PROFILE`). Missing credentials leave the files unsigned. The build job has
`contents: read`; only the explicitly requested release job has `contents: write`.

## Readiness and local packaging

[Readiness (Windows)](../.github/workflows/readiness-windows.yml) and
[Windows ARM64 packaging readiness](../.github/workflows/platform-windows-arm64-readiness.yml)
remain separate manual checks. The Windows measurement checks out current `main` and commits
`docs/readiness.md` with the source SHA in its report. Neither check starts after a push, tag,
or another workflow. Starting a platform build does not start readiness checks.

Local packaging commands are unchanged:

```sh
# On the matching host, with that platform's build tools installed:
pwsh packaging/windows/package.ps1 -Arch x64
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
| 2026-11-12 | minor | Five independent manual build workflows with selected source refs, commit provenance and artifacts; removed automatic tag triggers from the existing workflows; documented explicit x64 releases and platform limits. |
