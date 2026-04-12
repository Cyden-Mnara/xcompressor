# Releasing

## Overview

`xcompressor` publishes desktop releases from Git tags that match `v*`.

The release workflow:

- bundles FFmpeg and FFprobe into the desktop app
- builds signed Tauri bundles
- publishes release assets to GitHub Releases
- publishes Tauri updater metadata so installed apps can detect new versions

## 1. Generate updater signing keys

Install the Tauri CLI if needed:

```bash
cargo install tauri-cli --version "^2.0" --locked
```

Generate a signing keypair:

```bash
cargo tauri signer generate -w ~/.tauri/xcompressor-updater.key
```

This command will:

- create a private key file
- print the public key
- ask for a password to protect the private key

## 2. Configure GitHub secrets and variables

Add these repository secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
  - set this to the full contents of `~/.tauri/xcompressor-updater.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - set this to the password you entered while generating the key

Add this repository variable:

- `TAURI_UPDATER_PUBLIC_KEY`
  - set this to the public key printed by `cargo tauri signer generate`

## 3. Bump the app version

Set the version with:

```bash
pnpm version:set 0.1.1
```

This updates:

- [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json)
- [src-tauri/Cargo.toml](../src-tauri/Cargo.toml)
- [src-tauri/Cargo.lock](../src-tauri/Cargo.lock)

## 4. Create a release tag

Commit the version bump, then create and push a matching tag:

```bash
git tag v0.1.1
git push origin main --tags
```

Pushing the tag triggers:

- [.github/workflows/publish-release.yml](../.github/workflows/publish-release.yml)

## 5. Verify the release

After the workflow finishes, verify:

- the GitHub Release exists for the tag
- release assets were uploaded
- `latest.json` is attached to the release
- the installed app can detect the new version from the in-app updater panel

## Notes

- Development builds still fall back to `ffmpeg` and `ffprobe` on the system `PATH`.
- Release builds bundle FFmpeg through CI before Tauri packaging.
- If you rotate updater keys later, existing installed apps signed with the old public key will not trust updates signed by the new key unless you plan that migration explicitly.
