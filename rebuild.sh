#!/bin/bash

# Variables for paths
BUILD_PATH="/etc/ruche"
SERVICE_NAME="ruche.service"

# Pull latest code
git pull

# Set environment variable for Leptos optimization
export LEPTOS_TAILWIND_VERSION=v3.4.14

# Build assets and application
cargo run --release --bin asset-generation
cargo leptos build --release

# Stop the service
systemctl stop "$SERVICE_NAME"

# Copy the new binary
rm -rf "$BUILD_PATH/target/release/ruche-release"
cp -nf "$BUILD_PATH/target/release/ruche" "$BUILD_PATH/target/release/ruche-release"

# Start the service and follow logs
systemctl start "$SERVICE_NAME"
journalctl --follow -u "$SERVICE_NAME"