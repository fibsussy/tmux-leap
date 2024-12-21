#!/bin/bash

path=$(realpath ./__tmp_build)
mkdir -p $path
pushd $path
wget "https://raw.githubusercontent.com/Fibalious/jumper/refs/heads/main/PKGBUILD"
makepkg -si --noconfirm
popd
rm -rf $path
