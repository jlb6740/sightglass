FROM ubuntu:bionic
ENV RUST_VERISON 1.44.1

# Get some prerequisites
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	apt-utils build-essential gpg-agent \
	curl ca-certificates wget software-properties-common \
	psmisc lsof git nano zlib1g-dev libedit-dev time

# Bionic does not carry a recent enough cmake needed for wamr but upgrading
# to Focal causes other build issues so the straight forward solution is to just
# install a version of cmake that is recent enough from a separate repository
RUN wget -qO - https://apt.kitware.com/keys/kitware-archive-latest.asc | apt-key add -
RUN apt-add-repository 'deb https://apt.kitware.com/ubuntu/ bionic main'
RUN apt-get update
RUN apt-get install --yes cmake


# Install LLVM-10
RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|apt-key add -
RUN add-apt-repository "deb http://apt.llvm.org/bionic/ llvm-toolchain-bionic-10 main"
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	clang-10 lldb-10 lld-10
RUN ln -s /usr/lib/llvm-10/bin/wasm-ld /usr/bin/wasm-ld


# Install Wasi-11
RUN git clone --recurse-submodules https://github.com/CraneStation/wasi-libc.git wasilibc \
	&& cd wasilibc \
	&& make WASM_CC="/usr/bin/clang-10"  WASM_AR="/usr/bin/llvm-ar-10" WASM_NM="/usr/bin/llvm-nm-10"
RUN wget -q https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-11/libclang_rt.builtins-wasm32-wasi-11.0.tar.gz -O - | tar -xz \
	&& cp -r lib/wasi /usr/lib/llvm-10/lib/clang/10.*/lib/

ENV LD_LIBRARY_PATH=/usr/local/lib

# https://forge.rust-lang.org/infra/other-installation-methods.html
#RUN curl -sS -L -O https://static.rust-lang.org/dist/rust-${RUST_VERISON}-x86_64-unknown-linux-gnu.tar.gz \
#	&& tar xzf rust-${RUST_VERISON}-x86_64-unknown-linux-gnu.tar.gz \
#	&& cd rust-${RUST_VERISON}-x86_64-unknown-linux-gnu \
#	&& ./install.sh \
#	&& cd .. \
#	&& rm -rf rust-${RUST_VERISON}-x86_64-unknown-linux-gnu rust-${RUST_VERISON}-x86_64-unknown-linux-gnu.tar.gz

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-2020-07-28 -y
ENV PATH=/root/.cargo/bin:${PATH}


RUN curl -sL https://deb.nodesource.com/setup_14.x | bash
RUN apt-get install --yes nodejs
RUN npm install wasi
RUN node_location=$(which node) \
	&& mkdir -p vm_builds; cd vm_builds \
	&& cp ${node_location} ./ \
	&& cd ../../


RUN curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.34.0/install.sh | bash
ENV NVM_DIR /root/.nvm
ENV NODE_VERSION 14.6.0
RUN . ${NVM_DIR}/nvm.sh \
	&& nvm install ${NODE_VERSION} \
	&& nvm alias default ${NODE_VERSION} \
	&& nvm use default
ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH
RUN npm install wasi
RUN node -v
RUN npm -v


ENV PATH=/usr/local/bin:$PATH
CMD ["/bin/bash"]
WORKDIR "/sightglass_runner/"
