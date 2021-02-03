FROM ubuntu:bionic

COPY . /sg-static-view

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	apt-utils build-essential \
	curl ca-certificates \
	psmisc lsof nano

RUN curl -sL https://deb.nodesource.com/setup_14.x | bash
RUN apt-get install --yes nodejs

ENV LD_LIBRARY_PATH=/usr/local/lib

#RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-2020-07-28 -y
#ENV PATH=/root/.cargo/bin:${PATH}


WORKDIR "/sg-static-view"
RUN pwd && npm install -g yarn && ls -l && yarn install

#WORKDIR "/sg-history"
#RUN pwd && ls -l && cargo build

#WORKDIR "/"
