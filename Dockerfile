# Using this stage to build the binary
FROM buildpack-deps:buster as builder

RUN apt-get update && apt-get install --no-install-recommends -y \
    'build-essential' 'curl' 'ca-certificates' 'libssl-dev' && \
    rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/toolchain \
    PATH=$PATH:/toolchain/bin \
    RUSTUP_HOME=/toolchain

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
RUN chmod -R 777 /toolchain
WORKDIR /app

# To properly cache dependencies
COPY Cargo.toml Cargo.lock /app/
COPY driver/Cargo.toml /app/driver/Cargo.toml
RUN mkdir -p driver/src/ /plugin && touch driver/src/main.rs && (cargo build --release || true)

# Full build
COPY driver/ /app/driver/
RUN cargo build --release
