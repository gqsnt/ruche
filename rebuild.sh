#!/bin/bash
systemctl stop leptos_broken_gg.service;
git pull;
cargo leptos build --release;
systemctl start leptos_broken_gg.service;