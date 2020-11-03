#!/bin/bash
ORIG_CWD="$(pwd)/"
SCRIPT_LOC="$(realpath $(dirname ${BASH_SOURCE:-$0}))"
NODE_ROOT="${SCRIPT_LOC}/.."
SIGHTGLASS_ROOT="${SCRIPT_LOC}/../../.."
source ${SIGHTGLASS_ROOT}/config.inc

WASM_ENTRY=-DWASM_ENTRY
WASI_NO_SUPPORT="-DNO_WASI_SUPPORT"
WASI_CFLAGS="--sysroot=${WASI_SYSROOT} --target=wasm32-unknown-wasi"
TEMPLATE_SUB="REPLACE_ME_WITH_WASM_FILE_PATH"

#Prepare shootout
mkdir -p ${SCRIPT_LOC}/benchmark; cd ${SCRIPT_LOC}/benchmark
cp ${SIGHTGLASS_ROOT}/benchmarks/shootout_wasm/* .


#Build shootout
for wasmfile in ./*.wasm; do
    #echo ${WASM_CC} ${WASM_ENTRY} ${WASI_CFLAGS} ${WASI_NO_SUPPORT} ${COMMON_CFLAGS} -c $cfile -o $(basename -s .c "$cfile").wasm.o
    #${WASM_CC} ${WASM_ENTRY} ${WASI_CFLAGS} ${WASI_NO_SUPPORT} ${COMMON_CFLAGS} -c $cfile -o $(basename -s .c "$cfile").wasm.o
    #echo ${WASM_CC} ${WASM_ENTRY} ${WASI_CFLAGS} ${WASI_NO_SUPPORT} ${COMMON_CFLAGS} $(basename -s .c "$cfile").wasm.o -o $(basename -s .c "$cfile").wasm -nostartfiles -Wl,--no-entry -Wl,--export-all -Wl,--gc-sections -Wl,--strip-all
    #${WASM_CC} ${WASM_ENTRY} ${WASI_CFLAGS} ${WASI_NO_SUPPORT} ${COMMON_CFLAGS} $(basename -s .c "$cfile").wasm.o -o $(basename -s .c "$cfile").wasm -nostartfiles -Wl,--no-entry -Wl,--export-all -Wl,--gc-sections -Wl,--strip-all
    echo cp ${SCRIPT_LOC}/node.js.template $(basename -s .wasm "$wasmfile").js
    cp ${SCRIPT_LOC}/node.js.template $(basename -s .wasm "$wasmfile").js
    echo sed -i "s~${TEMPLATE_SUB}~\\/${SCRIPT_LOC}/benchmark/$(basename -s .wasm ${wasmfile}).wasm~g" $(basename -wasm .wasm "$wasmfile").js
    sed -i "s~${TEMPLATE_SUB}~\\/${SCRIPT_LOC}/benchmark/$(basename -s .wasm ${wasmfile}).wasm~g" $(basename -s .wasm "$wasmfile").js
done

#Build implementation.so
mkdir -p ${SCRIPT_LOC}/bin; cd ${SCRIPT_LOC}/bin
echo ${NODE_ROOT}
>&2 echo ${CC} ${COMMON_CFLAGS} -DWORKLOAD_LOCATION=${SCRIPT_LOC}/benchmark -DVM_LOCATION=${NODE_ROOT}/build/node -shared -o implementation.so ../wrapper.c
${CC} -fPIC ${COMMON_CFLAGS} -DWORKLOAD_LOCATION=${SCRIPT_LOC}/benchmark -DVM_LOCATION=${NODE_ROOT}/build/node -shared -o implementation.so ../wrapper.c

cd ${ORIG_CWD}