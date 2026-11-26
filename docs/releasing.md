# Building and releasing Doc4Me

> **Last reviewed** 2026-11-25 · **Last updated** 2026-11-25 · The manual Release workflow reports signing results and can publish ordinary builds for platforms whose signing credentials are missing.

## Build a chosen commit

All current workflows start only from `workflow_dispatch` (the platform builds can also be called by
the Release workflow). Pushing code or a tag does not start a build, deployment, or release. The
platform build workflows upload artifacts, the separate Pages workflow publishes the selected Web App
to GitHub Pages, and the [Release](../.github/workflows/release.yml) workflow publishes a GitHub
Release only when someone runs it (see below).

Doc4Me names its builds by the **commit** they were built from, not by a version number. `doc4me --version`, `doc4me-cli --version`, the MCP server's `serverInfo.version`
and the web user agent show the short commit (8 hex digits, `dev` for a local build without
`DOC4ME_BUILD_SHA`), and package files are named `doc4me-<commit>-<platform>-<arch>.<ext>` unless
another **file-name** is chosen (below).

A build may also be given a **version number** in the workflow form's **version** field: `1.2.3` or
`1.2.3-rc.1` (a leading `v` is dropped; major and minor up to 255 and patch up to 65535, the MSI
limits). The workflow checks it and passes it to the build as `DOC4ME_VERSION`
(`crates/engine/src/build_info.rs`): `--version` prints `doc4me 1.2.3 (abcdef12)`, the MCP server and user agent give `1.2.3`, the
installers' numeric version fields below take it, and the artifact holds `VERSION.txt`.

Two fields choose whether, and how, the version number and the commit appear (each defaults to
`commit` in the build workflows; the version choices need a version number, otherwise the run stops):

| Choice | **about**: line under the name in the About dialog | **file-name**: download files |
|---|---|---|
| `commit` | `Commit abcdef12` | `doc4me-abcdef12-windows-x64.msi` |
| `version` | `Version 1.2.3` | `doc4me-1.2.3-windows-x64.msi` |
| `version-commit` | `Version 1.2.3 (abcdef12)` | `doc4me-1.2.3-abcdef12-windows-x64.msi` |
| `none` | nothing | `doc4me-windows-x64.msi` |

File names cover the MSI, portable ZIP, DMG, CLI ZIP (`doc4me-cli-…`), AppImage/zsync, DEB, RPM,
tar.gz, Flatpak, Web ZIP (`doc4me-web-…`) and the folders inside the archives. The commit is always
recorded in `SOURCE_COMMIT.txt`, whichever is chosen. The Pages workflow has no download files, so it
has only **about**. A local build without `DOC4ME_BUILD_SHA` shows `development build` for `commit`.

The workflow checks both values and passes them as `DOC4ME_ABOUT` (read by `build_info::about_line`)
and `DOC4ME_FILE_NAME` (read by `packaging/env.sh` and `packaging/windows/package.ps1`, which build
the file names from the commit and `DOC4ME_VERSION`; a local packaging run with a version choice and
no `DOC4ME_VERSION` uses the `Cargo.toml` version).

Older source commits: every build workflow first checks out
[`tools/about_label.py`](../tools/about_label.py) and the interface catalogs from its own revision
and runs it on the selected source. A source that does not read `DOC4ME_ABOUT` gets its About line
replaced by code that reads the same variables (`DOC4ME_ABOUT`, `DOC4ME_VERSION`, `DOC4ME_BUILD_SHA`),
and the strings it uses are added to its catalogs where missing, so the About dialog of a build from
an older commit shows the choice, translated. Newer sources are left unchanged; the run stops if the
dialog code is not recognised (checked against the About code of every era since the first import).
File names of older sources: sources that predate `DOC4ME_FILE_NAME` keep their own names (a warning
says so), and those that know only `commit`/`version` get `version` for `version-commit` and `commit`
for `none`, with a warning. Older sources' `--version` and MCP version are not changed.

Without a version, the numeric fields that installers require keep the `[workspace.package] version` from
`Cargo.toml`: the MSI ProductVersion, the macOS `CFBundleShortVersionString`/`CFBundleVersion`,
the DEB/RPM package version, the AppStream release and the numeric Windows file version. The MSI
records the commit in its summary and in Programs and Features (`ARPCOMMENTS`), and `doc4me.exe`
carries it as its file and product version strings.

1. Open [Actions](https://github.com/veevie55/doc4me/actions) and choose a workflow.
2. Choose **Run workflow**, keeping **Use workflow from: main**. This selects the current workflow
   definition, independently of the source revision to build.
3. Set **ref** to `main` for current code, or a commit SHA (or branch) from this repository, such
   as the commit a published build came from. Optionally type a **version** and choose
   **file-name** and **about** (`commit`, `version`, `version-commit` or `none`). For macOS, choose `universal`, `aarch64`, or `x86_64` as well.
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

These seven platform-build jobs have read-only repository permissions and do not receive signing credentials;
the version typed in a form reaches the scripts only through an environment variable after the check above.
Third-party actions are pinned to commits. Linux packaging tools (nFPM 2.49.0 and appimagetool
1.9.1) and Trunk 0.22.1 are pinned and their downloads checked against SHA-256 digests.
Standalone Windows builds produce unsigned packages. Standalone macOS builds use ad-hoc signing
without notarization. The BSD build targets FreeBSD.

The FreeBSD VM installs Rust, LLVM/libclang, CMake, Ninja, ALSA, and window-system libraries,
then builds the app and CLI natively. Its archive needs the matching system libraries on the
destination machine; it is not a statically linked universal BSD binary.

The Web App ZIP can be extracted and served over HTTP(S), or published with the Pages workflow above.
See [packaging/web/README.md](../packaging/web/README.md) for MIME types, paths, and hosting options.

## Publishing a GitHub Release

[Release](../.github/workflows/release.yml) tags and publishes a Release. Like every workflow here it
starts only from **Run workflow** (keep **Use workflow from: main**). Its form:

| Field | Meaning |
|---|---|
| **version** | Required. `1.2.3` or `1.2.3-rc.1`, checked as above. The tag is `v<version>`; the run stops if that tag or Release already exists. |
| **ref** | `main` for the latest code (default). With **sign** off, a commit SHA, tag or branch also works. Each build uses the same resolved source commit. With **sign** on, the workflow builds the `main` commit used at dispatch. |
| **windows** | The Windows builds: `x64` (default), `x64, arm64`, `x86`, `x86, x64`, `x86, arm64`, `x86, x64, arm64`, `arm64` or `none`. One choice instead of three boxes, because GitHub allows at most 10 fields in a form. |
| **linux**, **macos**, **web** | Tick the other platform builds (at least one platform overall). macOS builds the universal DMG. |
| **pages** | Build and deploy the Web App to GitHub Pages. Select **web** for a downloadable ZIP in the Release. If **sign** is enabled, Pages waits for the signing jobs, including their credential checks. |
| **file-name** | As in the table above; default `version`. |
| **about** | As in the table above; default `version-commit`. |
| **sign** | Sign selected Windows and macOS packages where credentials are configured. Defaults to off. To enable it, choose **Use workflow from: main**, enter `main` under **ref**, and select at least one Windows or macOS build. |

A version with a pre-release part (`1.2.0-rc.1`, the text after `-`) is published as a **pre-release**;
a plain `1.2.0` becomes the latest release (semantic versioning, instead of a separate field).

The run calls the chosen platform workflows above (`workflow_call`) with the resolved SHA, the
version and the file-name and about choices, so each package carries the version in its `--version` and installer fields, and its file names and About dialog are as chosen.

Leave **sign** off to publish unsigned Windows packages and ad-hoc signed macOS packages without
notarization. With **sign** on, a platform with missing required credentials uses those same build
artifacts. The Actions run and Release notes identify which platform was skipped. The workflow
still signs the other selected platform when its credentials are configured.

When the chosen builds, requested signing jobs and requested Pages deployment succeed, the `publish`
job (the only job with `contents: write`) checks that each release artifact's `SOURCE_COMMIT.txt`
and `VERSION.txt` match, refuses two builds producing the same
file name, adds `SHA256SUMS.txt` and `SOURCE_COMMIT.txt`, writes cumulative notes with
[`tools/release_notes.py`](../tools/release_notes.py) from that commit's `CHANGELOG.md` (its
`## [<commit>]` section, or else its `[Unreleased]` entries headed `[<version> · <commit>]`, then every
earlier release), and creates the tag on the resolved commit together with the Release. A failed build or
signing job blocks the GitHub Release and its tag. Notes that exceed GitHub's 125,000-byte limit keep each
release's headings and top-level entries, cut to 160, 120, 100, 80 or 60 characters as needed, with a
link to the full section.

Before running it:

1. Run `cargo xtask ci`, the normally ignored interface audit
   (`cargo test -p doc4me-ui-egui i18n -- --include-ignored`), and `cargo audit`; push the source.
2. Optionally move the `[Unreleased]` entries in `CHANGELOG.md` under a heading
   `## [<commit>] - YYYY-MM-DD` (CLAUDE.md); otherwise the workflow uses them as they are.
   Headings of earlier releases keep their version numbers (history is not edited).

To publish by hand instead, run the platform workflows with the same **ref**, **version**, **file-name** and **about**, then
publish their Artifacts with notes from `tools/release_notes.py`.

`cargo xtask version` remains for the numeric installer fields of builds without a version; change it
only when installers must see a higher number.

### Code signing credentials

1. In **Settings → Environments**, create `release-signing`. Add required reviewers and restrict
   deployments to the `main` branch. Store the credentials below as secrets in that environment
   for the targets you plan to sign.
2. Run **Release** from `main` with **ref** set to `main`, choose Windows or macOS, and enable
   **sign**. The prepare job's summary identifies the source commit for review.
3. Approve the `release-signing` jobs after checking the source commit. The jobs run on fresh
   runners, using packaging helpers from the workflow revision. They check the source commit and
   version of the downloaded build artifacts and process them as data. The source build jobs run
   without signing credentials or the shared Rust cache. The Release's `SHA256SUMS.txt` covers the
   files actually published.

If a required secret is missing, the platform's signing job reports the skip after gaining access
to the environment. If configured credentials are rejected, signature checks fail, or Apple rejects
notarization, the workflow stops before publishing. The completed build artifacts remain available
under the Actions run.

**Windows x86, x64 and ARM64.** Use one of these signing providers.

- A trusted Authenticode signing certificate and private key exported as a PFX. Set
  `WINDOWS_CERTIFICATE` to the base64-encoded PFX and `WINDOWS_CERTIFICATE_PASSWORD` to its password.
- Microsoft Artifact Signing (formerly Trusted Signing). Provide `AZURE_TENANT_ID`,
  `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, and
  `AZURE_CERT_PROFILE`. Grant the application identity permission to sign with the selected
  public trust certificate profile.

The signing job applies timestamped Authenticode signatures to `doc4me.exe` and `doc4me-cli.exe`,
includes the signed executables in the MSI and portable ZIP, and signs the MSI itself. On Windows,
you can run `Get-AuthenticodeSignature` on the MSI and on the extracted executables to check their
publisher and signature status.

**macOS.** Set `APPLE_CERTIFICATE` to a base64-encoded Developer ID Application `.p12` that includes
the private key, and set `APPLE_CERTIFICATE_PASSWORD` to its password. Provide the Apple Account
`APPLE_ID`, its app-specific `APPLE_PASSWORD`, and the Developer Program `APPLE_TEAM_ID` for
notarization. The runner generates a password for the temporary signing keychain unless you set
`KEYCHAIN_PASSWORD`.

The signing job signs the app and CLI with the Developer ID identity, notarizes the DMG and CLI ZIP,
and staples the notarization ticket to the DMG. On macOS, `xcrun stapler validate <file.dmg>` checks
the DMG ticket and `codesign --verify --strict <path-to-doc4me-cli>` checks the extracted CLI signature.

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
| 2026-11-25 | minor | Documented publication with missing signing credentials and reporting for each platform. |
| 2026-11-25 | minor | Documented the manual Release signing option, environment setup and signature verification. |
| 2026-11-25 | minor | Added x86 and GitHub Pages to the Release options. Documented Windows/macOS signing inputs and a protected, artifact-only signing design; build jobs do not receive signing secrets while source code from the selected ref runs. |
| 2026-11-17 | minor | Build, Pages and Release workflows take an **about** choice and a wider **file-name** choice (commit, version, version-commit, none) to show or hide the version and the commit; the Release form's three Windows boxes became one **windows** choice (10-field limit). |
| 2026-11-17 | minor | The About dialog follows the file-name choice (version, or commit; the Pages workflow: the version if typed), also when an older commit is built: the build workflows patch its About line with `tools/about_label.py`. |
| 2026-11-17 | minor | Build workflows take an optional version number (About, `--version`, installer fields) and a **file-name** choice (download files named by commit or by version); a manual Release workflow tags `v<version>` and publishes a Release from `main` or a chosen commit with the ticked platforms (pre-release when the version has a pre-release part). |
| 2026-11-16 | minor | Builds are named by commit instead of a version number (`--version`, MCP, user agent, package file names; the About dialog shows neither); Releases are published by hand; removed the deleted x64 installer and ARM64 readiness workflows from this page. |
| 2026-11-15 | minor | Windows packages and readiness use a shared pinned OFL font set; the preparation script verifies Google Fonts hashes for Overpass and Noto Sans Ethiopic before embedding and packaging licences. |
| 2026-11-15 | minor | Added manual Windows x86 and x64 artifact workflows; both build MSI and portable ZIP from a selected ref and include source commit provenance. x86 uses the x64-hosted x86 MSVC toolchain. Neither workflow publishes a Release. |
| 2026-11-15 | minor | Added a manual GitHub Pages workflow for the Web App. It builds a selected ref with pinned Trunk, validates the static package, and deploys only from the main workflow definition to the `github-pages` environment. |
| 2026-11-12 | minor | Five independent manual build workflows with selected source refs, commit provenance and artifacts; removed automatic tag triggers from the existing workflows; documented explicit x64 releases and platform limits. |
