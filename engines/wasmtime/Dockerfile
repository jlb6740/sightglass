FROM rust:1.60
ARG REPOSITORY=https://github.com/bytecodealliance/wasmtime/
ARG REVISION=main
ARG BUILD="cargo build -p wasmtime-bench-api"
ARG FLAGS="--release"

WORKDIR /usr/src
# Clone a single commit (actually, a "revision"--tags and branches work as well).


RUN git clone --recurse-submodule https://github.com/bytecodealliance/wasmtime/
WORKDIR /usr/src/wasmtime
RUN git checkout 5b09b7
RUN git submodule update --init
RUN cargo build -p wasmtime-bench-api --release
RUN mv target/release/libwasmtime_bench_api.so ./libengine.so



#RUN git init
#RUN git remote add origin https://github.com/bytecodealliance/wasmtime/
#RUN git fetch --depth 1 origin
#RUN git checkout FETCH_HEAD
#RUN git submodule update --init --depth 1
## Build the engine library.
#RUN cargo build -p wasmtime-bench-api --release
##RUN ls -l target/release/libwasmtime_bench_api.so
##RUN ls -l /
#RUN mv target/release/libwasmtime_bench_api.so ./libengine.so
