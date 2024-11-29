#!/bin/bash

# Variables for paths
BUILD_PATH="/etc/broken-gg"
RELEASE_PATH="/etc/broken-gg-release"
SERVICE_NAME="broken_gg.service"

# Pull latest code
git pull

# Set environment variable for Leptos optimization
export LEPTOS_WASM_OPT_VERSION=version_119

# Build assets and application
cargo run --release --bin asset-generation
cargo leptos build --release

# Stop the service
systemctl stop "$SERVICE_NAME"

# Remove old release and recreate necessary directories
rm -rf "$RELEASE_PATH"
mkdir -p "$RELEASE_PATH/target/release"
mkdir -p "$RELEASE_PATH/target/site"
mkdir -p "$RELEASE_PATH/signed_certs"

# Copy files to the release path
cp -nf "$BUILD_PATH/target/release/broken-gg" "$RELEASE_PATH/target/release/broken-gg"
cp -nfR "$BUILD_PATH/target/site/"* "$RELEASE_PATH/target/site/"
cp -nf "$BUILD_PATH/.env" "$RELEASE_PATH/.env"
cp -nf "$BUILD_PATH/signed_certs/"* "$RELEASE_PATH/signed_certs/"

# Start the service and follow logs
systemctl start "$SERVICE_NAME"
journalctl --follow -u "$SERVICE_NAME"