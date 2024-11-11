#!/bin/bash
git pull;
cargo leptos build --release;
systemctl restart leptos_broken_gg.service;
journalctl --follow -u leptos_broken_gg.service;