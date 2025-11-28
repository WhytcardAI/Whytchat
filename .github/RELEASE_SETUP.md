# Release Setup Guide

This document explains how to set up the release workflow for WhytChat.

## Required GitHub Secrets

Before running the release workflow, you need to configure the following secrets in your GitHub repository settings:

### 1. `TAURI_SIGNING_PRIVATE_KEY` (Optional but Recommended)

Used for signing Tauri updater bundles. Generate a key pair with:

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/whytchat.key
```

Then add the **private key** content to this secret.

### 2. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (Optional)

The password used when generating the signing key (if any).

## How to Create a Release

### Option 1: Tag-based Release

1. Create and push a tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. The release workflow will automatically trigger.

### Option 2: Manual Release

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter the version (e.g., `v1.0.0`)
4. Click **Run workflow**

## Pre-flight Validation

The release workflow includes a validation job that:
- ✅ Checks code formatting (`cargo fmt`)
- ✅ Runs ESLint on frontend code
- ✅ Runs Clippy for Rust linting
- ✅ Executes all tests
- ✅ Builds the frontend

Only if validation passes will the multi-platform builds begin.

## Build Artifacts

The workflow produces the following artifacts:

| Platform | Artifacts |
|----------|-----------|
| Windows | `.exe` (NSIS installer), `.msi` (WiX installer) |
| Linux | `.AppImage`, `.deb` |
| macOS Intel | `.dmg` |
| macOS ARM | `.dmg` |

## Troubleshooting

### Build Fails on Specific Platform

Check the workflow logs for the specific platform. Common issues:
- Missing system dependencies
- Protobuf compiler not found
- Signing key issues

### Validation Job Fails

Run the following locally to identify issues:
```bash
# Check formatting
cargo fmt --manifest-path apps/core/Cargo.toml -- --check

# Run ESLint
npm run lint:js

# Run Clippy
cargo clippy --manifest-path apps/core/Cargo.toml -- -D warnings

# Run tests
cargo test --manifest-path apps/core/Cargo.toml
```
