ARG CROSS_DEB_ARCH
FROM ubuntu:18.04

COPY additional_sources.list /tmp/additional_sources.list

RUN cat /tmp/additional_sources.list > /etc/apt/sources.list && rm /tmp/additional_sources.list && \
    dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes pkg-config wget unzip ca-certificates curl git build-essential gcc-aarch64-linux-gnu libudev-dev libudev-dev:arm64 && \
    rm -rf /var/lib/apt/lists/*

RUN TEMP_DIR=`mktemp -d`; \
    cd $TEMP_DIR && \
    wget https://github.com/protocolbuffers/protobuf/releases/download/v28.2/protoc-28.2-linux-x86_64.zip && \
    unzip protoc-28.2-linux-x86_64.zip && \
    mv bin/* /usr/bin && \
    mv include/* /usr/include && \
    cd / && \
    rm -rf $TEMP_DIR
