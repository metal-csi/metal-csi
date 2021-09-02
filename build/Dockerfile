# Using this stage to build the binary
FROM buildpack-deps:buster as builder

RUN apt-get update && apt-get install -y \
    'build-essential' 'curl' 'ca-certificates' 'gcc-aarch64-linux-gnu' 'gcc-arm-linux-gnueabihf' 'lzip' && \
    rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/toolchain \
    PATH=$PATH:/toolchain/bin \
    RUSTUP_HOME=/toolchain

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -t aarch64-unknown-linux-gnu -t armv7-unknown-linux-gnueabihf -y
RUN chmod -R 777 /toolchain
WORKDIR /app

# To properly cache dependencies
COPY Cargo.toml Cargo.lock /app/
COPY driver/Cargo.toml /app/driver/Cargo.toml
COPY .cargo/ /app/.cargo/
