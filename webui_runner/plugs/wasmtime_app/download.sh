#!/bin/bash

>&2 echo "wasmtime_app build"
mkdir -p build; cd build
if [ ! -d wasmtime ]; then
    git clone https://github.com/jlb6740/wasmtime.git --branch add-arm-support wasmtime
fi
cd wasmtime
git clean -fd
git submodule foreach --recursive git clean -fd
git reset --hard
git submodule init && git submodule update
git submodule foreach --recursive git submodule init
git submodule foreach --recursive git submodule update
>&2 echo "wasmtime_app download"
cargo build --release
cd ../../
