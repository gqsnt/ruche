#!/bin/bash

# Variables for paths
BUILD_PATH="/etc/ruche"
SERVICE_NAME="ruche.service"

# Pull latest code
git pull


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