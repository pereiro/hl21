# 1: Build the binary
FROM rust:1.50.0 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    apt-get install -y libssl-dev && \
    apt-get install -y ca-certificates && \
    apt-get install -y pkg-config && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl

ENV OPENSSL_DIR=/usr/local

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
#RUN USER=root cargo new hl21
WORKDIR /usr/src/hl21
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN mkdir .cargo
RUN cargo vendor > .cargo/config
#RUN cargo install --target x86_64-unknown-linux-musl --path .

# 1c: Build the binary using the actual source code
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 2: Copy the binary and extra files ("static") to an empty Docker image
FROM alpine
COPY --from=builder /usr/local/cargo/bin/hl21 .
USER 1000

ENV SEARCH_BINARY_ENABLED=false
ENV SEARCH_INITIAL_ARRAY_SIZE=511
ENV SEARCH_MIN_AMOUNT=22
ENV SEARCH_TO_FLAT_THRESHOLD=31
ENV SEARCH_FLAT_SIZE=3

ENV DIGGER_MIN_DEPTH=2
ENV DIGGER_MAX_DEPTH=10
ENV DIGGER_MIN_DEPTH_PROBABILITY=5

ENV ATTORNEY_LICENSE_MIN_COST=1
ENV ATTORNEY_LICENSE_MAX_COST=1
ENV ATTORNEY_FREE_LICENSE_PROBABILITY=100
ENV ATTORNEY_HTTP_TIMEOUT_MS=150

ENV ACCOUNTANT_HTTP_TIMEOUT_MS=550
ENV HTTP_TIMEOUT_MS=550

ENV SEARCH_EXPLORERS_NUM=16
ENV ATTORNEYS_NUM=16
ENV DIGGERS_NUM=8
ENV ACCOUNTANT_NUM=8

ENV MAX_RPS=1500
ENV EXPLORE_PHASE1_RPS=950
ENV ACCOUNTANT_PHASE1_RPS=1
ENV DIGGER_PHASE1_RPS=500
ENV ATTORNEY_PHASE1_RPS=350

ENV ENABLE_PHASED=true
ENV PHASE2_START=350

ENV EXPLORE_PHASE2_RPS=1
ENV ACCOUNTANT_PHASE2_RPS=200
ENV DIGGER_PHASE2_RPS=1
ENV ATTORNEY_PHASE2_RPS=1

ENV AREA_CHAN_CAP=5
ENV TILE_CHAN_CAP=5
ENV LICENSE_CHAN_CAP=30
ENV EMPTY_LICENSE_CHAN_CAP=25
ENV TREASURE_CHAN_CAP=40000

CMD ["./hl21"]