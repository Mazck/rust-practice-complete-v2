FROM rust:1.89-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY modules_lab ./modules_lab
COPY cargo_lab ./cargo_lab
COPY network_lab ./network_lab
COPY production_lab ./production_lab
RUN cargo build --release -p network-lab --bin network-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/network-server /usr/local/bin/network-server
ENV RUST_LOG=info
ENV HTTP_ADDR=0.0.0.0:3000
EXPOSE 3000
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/network-server"]
