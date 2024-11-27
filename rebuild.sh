#!/bin/bash
git pull;
export LEPTOS_WASM_OPT_VERSION=version_119;
cargo leptos build --release;
systemctl stop leptos_broken_gg.service;
rm -rf /etc/leptos-broken-gg-release; ## RM -RF USED !!! Remove the old release
mkdir -p /etc/leptos-broken-gg-release/target/release;
mkdir -p /etc/leptos-broken-gg-release/target/site;
mkdir -p /etc/leptos-broken-gg-release/signed_certs;
cp -nf /etc/leptos-broken-gg/target/release/leptos-broken-gg /etc/leptos-broken-gg-release/target/release/leptos-broken-gg;
cp -nfR /etc/leptos-broken-gg/target/site/* /etc/leptos-broken-gg-release/target/site/;
cp -nf /etc/leptos-broken-gg/.env /etc/leptos-broken-gg-release/.env;
cp -nf /etc/leptos-broken-gg/signed_certs/* /etc/leptos-broken-gg-release/signed_certs/;
systemctl start leptos_broken_gg.service;
journalctl --follow -u leptos_broken_gg.service;