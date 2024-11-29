#!/bin/bash
git pull;
export LEPTOS_WASM_OPT_VERSION=version_119;
cargo leptos build --release;
systemctl stop broken_gg.service;
rm -rf /etc/broken-gg-release; ## RM -RF USED !!! Remove the old release
mkdir -p /etc/broken-gg-release/target/release;
mkdir -p /etc/broken-gg-release/target/site;
mkdir -p /etc/broken-gg-release/signed_certs;
cp -nf /etc/broken-gg/target/release/broken-gg /etc/broken-gg-release/target/release/broken-gg;
cp -nfR /etc/broken-gg/target/site/* /etc/broken-gg-release/target/site/;
cp -nf /etc/broken-gg/.env /etc/broken-gg-release/.env;
cp -nf /etc/broken-gg/signed_certs/* /etc/broken-gg-release/signed_certs/;
systemctl start broken_gg.service;
journalctl --follow -u roken_gg.service;