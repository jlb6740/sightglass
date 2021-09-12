FROM rust:1.54

COPY libengine /usr/src/native
WORKDIR /usr/src/native
RUN cargo build --release
RUN mv target/release/libnative_bench_api.so /libnative_bench_api.so
RUN cp /libnative_bench_api.so /libengine.so
#WORKDIR /usr/src/wasmtime
#RUN git checkout ${REVISION} && git submodule update --init
#RUN cargo build -p wasmtime-bench-api --release
#RUN mv target/release/libnative_bench_api.so /libengine.so
