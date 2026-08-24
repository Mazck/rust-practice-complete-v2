# Rust Practice Solutions

Đây là workspace lời giải đi kèm `rust-practice-workbook.md`.

## Kiểm tra toàn bộ

```bash
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo fmt --all -- --check
```

## Chạy từng package

```bash
cargo run -p modules-lab
cargo run -p cargo-lab
APP_VERSION=1.2.3 cargo run -p cargo-lab --no-default-features --features compact
HTTP_ADDR=127.0.0.1:3000 cargo run -p network-lab --bin network-server
```

## Chạy network examples

```bash
cargo run -p network-lab --example http_fixture_server  # terminal 1
DEMO_HTTP_BASE_URL=http://127.0.0.1:3010 cargo run -p network-lab --example http_client  # terminal 2
cargo run -p network-lab --example udp_demo
cargo run -p network-lab --example tcp_server                 # terminal 1
cargo run -p network-lab --example tcp_client                 # terminal 2
cargo run -p network-lab --example websocket_server           # terminal 1
cargo run -p network-lab --example websocket_client           # terminal 2
MQTT_HOST=127.0.0.1 MQTT_PORT=1883 cargo run -p network-lab --example mqtt_client
cargo test -p production-lab
cargo bench -p production-lab --bench telemetry_parse
```

## Production lab

`production_lab` có migration SQLite, transaction telemetry + outbox, deduplication theo `message_id`, Argon2 password hashing, JWT claims/issuer/expiry, property tests với Proptest và Criterion benchmark.

```bash
cargo test -p production-lab
cargo test -p production-lab --test properties
cargo bench -p production-lab --bench telemetry_parse
```

## MQTT broker local và Docker

```bash
docker compose up -d mqtt
docker compose ps
docker compose logs -f mqtt
./scripts/verify.sh
docker build -t rust-network-lab .
```

File `ops/mosquitto.conf` bật anonymous access chỉ cho local development. Production phải dùng TLS, credential, ACL topic, secret injection và policy firewall. Không đặt secret thật trong source, command history, image hoặc log.
