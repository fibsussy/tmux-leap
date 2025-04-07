#!/bin/bash

set -e
path=$(realpath /tmp/__leap_binary_tmp_build)

cleanup() {
    popd &>/dev/null || true
    rm -rf "$path"
}
trap cleanup EXIT INT TERM

mkdir -p "$path"
pushd "$path"

wget "https://raw.githubusercontent.com/fibsussy/leap/refs/heads/main/PKGBUILD"
makepkg -si --noconfirm
