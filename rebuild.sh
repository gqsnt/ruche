#!/bin/bash

# Variables for paths
BUILD_PATH="/etc/ruche"
RELEASE_PATH="/etc/ruche-release"
SERVICE_NAME="ruche.service"

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
mkdir -p "$RELEASE_PATH"
rm -rf "$RELEASE_PATH/target"
mkdir -p "$RELEASE_PATH/target/release"
mkdir -p "$RELEASE_PATH/target/site"

# Copy files to the release path
cp -nf "$BUILD_PATH/target/release/ruche" "$RELEASE_PATH/target/release/ruche"
cp -nfR "$BUILD_PATH/target/site/"* "$RELEASE_PATH/target/site/"
cp -nf "$BUILD_PATH/.env" "$RELEASE_PATH/.env"

# Start the service and follow logs
systemctl start "$SERVICE_NAME"
journalctl --follow -u "$SERVICE_NAME"