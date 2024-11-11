#!/bin/bash
git pull;
cargo leptos build --release;
systemctl stop leptos_broken_gg.service;
mkdir -p /etc/leptos-broken-gg-release/target/release;
mkdir -p /etc/leptos-broken-gg-release/target/site;
mv /etc/leptos-broken-gg/target/release/leptos-broken-gg /etc/leptos-broken-gg-release/target/release/leptos-broken-gg;
cp -nf /etc/leptos-broken-gg/.env /etc/leptos-broken-gg-release/.env;
mv /etc/leptos-broken-gg/target/site/* /etc/leptos-broken-gg-release/target/site/;
systemctl start leptos_broken_gg.service;
journalctl --follow -u leptos_broken_gg.service;