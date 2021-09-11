FROM rust:1.54
ARG REPOSITORY=https://github.com/bytecodealliance/wasmtime/
ARG REVISION=main

WORKDIR /usr/src
COPY engine.c ./
RUN echo "Some String" > /libengine.so
#WORKDIR /usr/src/wasmtime
#RUN git checkout ${REVISION} && git submodule update --init
#RUN cargo build -p wasmtime-bench-api --release
#RUN mv target/release/libnative_bench_api.so /libengine.so
