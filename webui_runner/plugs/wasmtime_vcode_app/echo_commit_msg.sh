ORIG_CWD="$(pwd)"
SCRIPT_LOC="$(dirname ${BASH_SOURCE:-$0})"
cd ${SCRIPT_LOC}/build/wasmtime_vcode/
echo $(git log -1 --pretty=format:"%cr - %s")
cd ${ORIG_CWD}