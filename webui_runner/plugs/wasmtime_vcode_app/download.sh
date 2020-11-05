#!/bin/bash

>&2 echo "wasmtime_vcode_app download"
mkdir -p build; cd build
if [ ! -d wasmtime_vcode ]; then
    git clone https://github.com/CraneStation/wasmtime.git wasmtime_vcode
fi
cd wasmtime_vcode
git clean -fd
git submodule foreach --recursive git clean -fd
git reset --hard
git submodule init && git submodule update
git submodule foreach --recursive git submodule init
git submodule foreach --recursive git submodule update
git pull

>&2 echo "wasmtime_vcode_app build"
cargo build --release --features experimental_x64
cd ../../
