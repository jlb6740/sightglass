FROM ubuntu:bionic
ENV RUST_VERISON 1.44.1

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	apt-utils build-essential gpg-agent \
	curl ca-certificates wget software-properties-common \
	psmisc lsof git nano zlib1g-dev libedit-dev

RUN wget -qO - https://apt.kitware.com/keys/kitware-archive-latest.asc | apt-key add -
RUN apt-add-repository 'deb https://apt.kitware.com/ubuntu/ bionic main'
RUN apt-get update
RUN apt-get install --yes cmake

RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|apt-key add -

RUN add-apt-repository "deb http://apt.llvm.org/bionic/ llvm-toolchain-bionic-10 main"

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	clang-10 lldb-10 lld-10

RUN ln -s /usr/lib/llvm-10/bin/wasm-ld /usr/bin/wasm-ld

RUN git clone --recurse-submodules https://github.com/CraneStation/wasi-libc.git wasi_libc \
	&& cd wasi_libc \
	&& make WASM_CC="/usr/bin/clang-10"  WASM_AR="/usr/bin/llvm-ar-10" WASM_NM="/usr/bin/llvm-nm-10"

RUN wget -q https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-11/libclang_rt.builtins-wasm32-wasi-11.0.tar.gz -O - | tar -xz \
	&& cp -r lib/wasi /usr/lib/llvm-10/lib/clang/10.*/lib/

ENV LD_LIBRARY_PATH=/usr/local/lib

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash
RUN apt-get install --yes nodejs
RUN npm install wasi

ENV PATH=/usr/local/bin:$PATH
ENV PATH=/root/.cargo/bin:$PATH

CMD ["/bin/bash"]

WORKDIR "/sightglass_runner/"
