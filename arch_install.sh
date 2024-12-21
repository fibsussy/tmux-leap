#!/bin/bash

set -e
path=$(realpath ./__tmp_build)

cleanup() {
    popd &>/dev/null || true
    rm -rf "$path"
}
trap cleanup EXIT INT TERM

mkdir -p "$path"
pushd "$path"

wget "https://raw.githubusercontent.com/Fibalious/jumper/refs/heads/main/PKGBUILD"
makepkg -si --noconfirm
