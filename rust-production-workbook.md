# Rust Production Workbook — Phần mở rộng thực chiến

> Tài liệu này nối tiếp `rust-practice-workbook.md`. Phần trước tập trung vào module, Cargo, async và protocol; phần này tập trung vào các vấn đề thường quyết định một service có thể vận hành thật hay chỉ chạy được trong demo.

## Cách sử dụng

Mỗi bài gồm bối cảnh, mục tiêu, yêu cầu, lời giải tham khảo, test case và phần mở rộng. Hãy tạo branch riêng cho từng bài. Không dùng secret thật, không kết nối broker/Database production và không chạy migration phá dữ liệu nếu chưa có backup.

| Nhóm | Bài | Kỹ năng |
|---|---:|---|
| Correctness | 38–41 | invariants, property testing, benchmark, fault injection |
| Persistence | 42–44 | SQLx, SQLite, migration, transaction, password hashing |
| Security | 45–48 | JWT, authorization, rate limit, secret redaction |
| Operations | 49–53 | tracing, metrics, MQTT broker, outbox, Docker, health check |
| Capstone | 54–57 | integration, threat model, load test, nghiệm thu production |

---

# Phần A — Correctness, concurrency và hiệu năng

## Bài 38 — Viết invariants trước khi viết implementation

### Bối cảnh

Một gateway nhận telemetry từ nhiều transport. Dữ liệu hợp lệ phải luôn thỏa bốn invariant: `device_id` không rỗng, `message_id` không rỗng, `value` hữu hạn và `unit` thuộc tập cho phép.

### Yêu cầu

Viết kiểu `Telemetry`, hàm `validate`, và test cho dữ liệu hợp lệ, chuỗi rỗng, `NaN`, `INFINITY`, unit lạ và message ID trùng.

### Lời giải

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Telemetry {
    pub device_id: String,
    pub message_id: String,
    pub value: f32,
    pub unit: String,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ValidationError {
    #[error("device_id is empty")]
    EmptyDevice,
    #[error("message_id is empty")]
    EmptyMessage,
    #[error("value is not finite")]
    NonFiniteValue,
    #[error("unit is not supported")]
    UnsupportedUnit,
}

impl Telemetry {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.device_id.trim().is_empty() {
            return Err(ValidationError::EmptyDevice);
        }
        if self.message_id.trim().is_empty() {
            return Err(ValidationError::EmptyMessage);
        }
        if !self.value.is_finite() {
            return Err(ValidationError::NonFiniteValue);
        }
        if !matches!(self.unit.as_str(), "C" | "%" | "Pa") {
            return Err(ValidationError::UnsupportedUnit);
        }
        Ok(())
    }
}
```

### Test case bắt buộc

```rust
#[test]
fn rejects_non_finite_value() {
    let value = Telemetry {
        device_id: "d-1".into(),
        message_id: "m-1".into(),
        value: f32::NAN,
        unit: "C".into(),
    };
    assert_eq!(value.validate(), Err(ValidationError::NonFiniteValue));
}
```

### Tiêu chí đạt

Implementation không panic khi input bất kỳ, lỗi được phân loại thay vì trả `String`, và mọi invariant có ít nhất một test tốt cùng một test xấu.

---

## Bài 39 — Property-based testing với Proptest

### Mục tiêu

Kiểm tra parser không panic với dữ liệu byte tùy ý, và mọi giá trị telemetry được chấp nhận đều hữu hạn.

### Manifest

```toml
[dev-dependencies]
proptest = "1.11"
```

### Lời giải

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parser_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        let _ = serde_json::from_slice::<Telemetry>(&bytes);
    }

    #[test]
    fn valid_values_are_finite(value in -1_000_000.0f32..1_000_000.0f32) {
        let telemetry = Telemetry {
            device_id: "d".into(),
            message_id: "m".into(),
            value,
            unit: "C".into(),
        };
        prop_assert!(telemetry.validate().is_ok());
        prop_assert!(telemetry.value.is_finite());
    }
}
```

Proptest sinh nhiều input và shrink input thất bại về trường hợp nhỏ nhất. Không dùng property test để thay toàn bộ test nghiệp vụ; dùng nó để bắt panic, violation invariant, parser edge case và lỗi boundary.

### Bài mở rộng

Sinh `message_id` dài 0–10.000 byte và đặt giới hạn kích thước. Viết property chứng minh payload vượt giới hạn luôn bị từ chối trước khi deserialize sâu.

---

## Bài 40 — Benchmark bằng Criterion

### Mục tiêu

Đo thời gian parse JSON và so sánh với parser thủ công. Không kết luận hiệu năng từ một lần chạy `Instant::now()`.

### Manifest

```toml
[dev-dependencies]
criterion = "0.8"

[[bench]]
name = "telemetry_parse"
harness = false
```

### Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_benchmark(c: &mut Criterion) {
    let payload = br#"{"device_id":"d-1","message_id":"m-1","value":21.5,"unit":"C"}"#;
    c.bench_function("parse telemetry json", |bench| {
        bench.iter(|| {
            let value: Telemetry = serde_json::from_slice(black_box(payload)).unwrap();
            black_box(value);
        });
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
```

Chạy:

```bash
cargo bench --bench telemetry_parse
```

Criterion cung cấp đo lường thống kê và báo cáo xu hướng thay vì chỉ một con số tức thời. [11] Benchmark phải chạy trên cùng workload, compiler profile và môi trường đủ ổn định; không benchmark cùng lúc với broker, build hoặc workload khác.

### Bài mở rộng

Đo ba biến thể: `serde_json::from_slice`, `serde_json::Value` và parser typed. Ghi nhận throughput, allocation nếu cần, và đặt ngưỡng regression trong CI.

---

## Bài 41 — Fault injection cho timeout và disconnect

### Yêu cầu

Tạo fake upstream có các mode `Success`, `Delay`, `Disconnect`, `BadJson`, `Status500`. Client phải trả error đúng loại và không chờ vô hạn.

### Mô hình

```rust
#[derive(Debug, Clone, Copy)]
pub enum FailureMode {
    Success,
    Delay,
    Disconnect,
    BadJson,
    Status500,
}
```

### Test timeout

```rust
#[tokio::test]
async fn delay_is_bounded() {
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        fake_upstream(FailureMode::Delay),
    ).await;

    assert!(result.is_err(), "operation must not wait forever");
}
```

### Bảng test

| Failure mode | Kết quả mong đợi | Có retry? |
|---|---|---:|
| Success | `Ok` | Không cần |
| Delay | `Timeout` | Có thể, nếu operation idempotent |
| Disconnect | `Transport` | Có thể |
| BadJson | `Decode` | Không retry mù |
| Status500 | `Http(500)` | Có backoff |

Fault injection nên dùng fixture local, không cần làm hỏng Internet thật. Mục tiêu là kiểm tra state machine và error mapping.

---

# Phần B — Persistence với SQLx và SQLite

## Bài 42 — SQLite pool và migration

### Manifest

```toml
[dependencies]
sqlx = { version = "0.9", default-features = false, features = ["sqlite", "runtime-tokio-rustls", "macros", "migrate"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

SQLx là async SQL toolkit, hỗ trợ SQLite/Postgres/MySQL, pool và query macros; async API cần runtime feature phù hợp. [8]

### Migration

```text
migrations/20240101000000_create_telemetry.sql
```

```sql
CREATE TABLE telemetry (
    device_id TEXT NOT NULL,
    message_id TEXT PRIMARY KEY,
    value REAL NOT NULL,
    unit TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_telemetry_device_created
ON telemetry(device_id, created_at);
```

### Lời giải

```rust
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn open_database(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

Chạy với database file:

```bash
DATABASE_URL=sqlite://telemetry.db cargo run
```

Test nên dùng `sqlite::memory:` và mỗi test tạo pool riêng. Khi dùng in-memory SQLite với nhiều connection, cần hiểu semantics của connection pool; cách đơn giản cho test nhỏ là `max_connections(1)` hoặc dùng file tạm.

---

## Bài 43 — Query typed và transaction

### Model

```rust
#[derive(Debug, sqlx::FromRow)]
pub struct TelemetryRow {
    pub device_id: String,
    pub message_id: String,
    pub value: f64,
    pub unit: String,
}
```

### Insert typed

```rust
pub async fn insert(
    pool: &sqlx::SqlitePool,
    value: &TelemetryRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO telemetry(device_id, message_id, value, unit)
           VALUES (?, ?, ?, ?)"#,
    )
    .bind(&value.device_id)
    .bind(&value.message_id)
    .bind(value.value)
    .bind(&value.unit)
    .execute(pool)
    .await?;
    Ok(())
}
```

### Transaction và outbox

```rust
pub async fn insert_with_outbox(
    pool: &sqlx::SqlitePool,
    value: &TelemetryRow,
    event_json: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO telemetry(device_id, message_id, value, unit) VALUES (?, ?, ?, ?)")
        .bind(&value.device_id)
        .bind(&value.message_id)
        .bind(value.value)
        .bind(&value.unit)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO outbox(message_id, payload, delivered) VALUES (?, ?, 0)")
        .bind(&value.message_id)
        .bind(event_json)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
```

Nếu insert telemetry thành công nhưng publish MQTT thất bại, outbox giữ event để worker gửi lại. Đây là cách tránh mất event giữa database và broker.

### Test transaction rollback

```rust
#[tokio::test]
async fn rollback_removes_partial_write() {
    let pool = open_database("sqlite::memory:").await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("CREATE TABLE t (id INTEGER PRIMARY KEY)")
        .execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO t(id) VALUES (1)")
        .execute(&mut *tx).await.unwrap();
    tx.rollback().await.unwrap();
}
```

Không nhúng raw user input vào SQL string. Dùng bind parameters; query macros có thể type-check SQL khi build với database metadata phù hợp. [8]

---

## Bài 44 — Password hashing với Argon2id

### Manifest

```toml
argon2 = "0.5"
```

### Lời giải

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &[u8]) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default().hash_password(password, &salt)?.to_string())
}

pub fn verify_password(password: &[u8], encoded: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded) else { return false; };
    Argon2::default().verify_password(password, &parsed).is_ok()
}
```

Argon2id là lựa chọn mặc định trong API ví dụ của crate; kết quả là PHC string chứa algorithm, version, parameters, salt và hash. [9] Không lưu plaintext password, không log password, không tự tạo salt cố định, và không dùng `hash_password_into` để thay thế password hashing nếu không hiểu rõ mục tiêu cryptographic key derivation.

### Test

```rust
#[test]
fn password_round_trip() {
    let encoded = hash_password(b"correct horse battery staple").unwrap();
    assert!(verify_password(b"correct horse battery staple", &encoded));
    assert!(!verify_password(b"wrong", &encoded));
}
```

### Production note

Cost parameters cần benchmark trên phần cứng triển khai và điều chỉnh theo latency budget. Không copy parameters từ một blog mà không đo. Khi nâng cost, user cũ có thể được rehash sau khi đăng nhập thành công.

---

# Phần C — Authentication và authorization

## Bài 45 — JWT claims có expiry và issuer

### Manifest

```toml
jsonwebtoken = "11"
serde = { version = "1", features = ["derive"] }
```

### Model

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iss: String,
}
```

### Nguyên tắc

JWT không mã hóa nội dung; thường chỉ ký để kiểm tra integrity. Không đặt password, secret hoặc dữ liệu nhạy cảm vào claims. Server phải validate signature, algorithm, `exp`, `iss`, `aud` nếu dùng, và quyền cần thiết.

### Lời giải khung

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub fn issue(claims: &Claims, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret))
}

pub fn verify(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.set_issuer(&["device-gateway"]);
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
}
```

Trong production, secret phải lấy từ secret manager/environment injection, không commit vào Git. Với hệ thống nhiều service, cân nhắc asymmetric signing và key rotation; service verify chỉ cần public key.

---

## Bài 46 — Axum authentication extractor

### Yêu cầu

Viết extractor hoặc middleware đọc `Authorization: Bearer <token>`, verify JWT và đưa `CurrentUser` vào handler.

### Lời giải khung bằng `FromRequestParts`

```rust
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: String,
    pub role: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let token = header.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
        let claims = verify(token, std::env::var("JWT_SECRET").unwrap().as_bytes())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        Ok(Self { user_id: claims.sub, role: claims.role })
    }
}
```

Không dùng `unwrap()` như trên trong production; config phải được validate lúc startup và secret phải được lưu trong state typed. Extractor chỉ xác thực identity; authorization kiểm tra role/resource ownership ở service layer.

### Authorization test matrix

| User | Resource | Kết quả |
|---|---|---|
| owner | own device | allow |
| owner | other tenant device | deny |
| admin | any allowed tenant device | allow theo policy |
| expired token | any | deny |
| wrong issuer | any | deny |
| malformed token | any | deny |

---

## Bài 47 — Rate limiting và backpressure

### Yêu cầu

Giới hạn mỗi client 10 request/giây và không để channel nội bộ tăng vô hạn.

### Thiết kế

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

#[derive(Clone)]
pub struct Limits {
    pub upstream: Arc<Semaphore>,
}

impl Limits {
    pub fn new(max_in_flight: usize) -> Self {
        Self { upstream: Arc::new(Semaphore::new(max_in_flight)) }
    }
}
```

Rate limit và concurrency limit khác nhau: rate limit giới hạn tốc độ theo thời gian, semaphore giới hạn số operation đồng thời. Cần xác định policy khi quá hạn: trả `429`, drop telemetry không quan trọng, hoặc ghi outbox để xử lý sau.

### Test

Tạo 20 task, semaphore 4, đo rằng không quá 4 task vào vùng downstream tại cùng thời điểm. Không đo bằng `println!`; dùng atomic counter và assert `max_seen <= 4`.

---

## Bài 48 — Secret redaction và security regression test

### Helper

```rust
pub fn redact_authorization(value: &str) -> String {
    if value.starts_with("Bearer ") {
        "Bearer [REDACTED]".to_owned()
    } else {
        "[REDACTED]".to_owned()
    }
}
```

### Test

```rust
#[test]
fn logs_do_not_contain_token() {
    let token = "Bearer secret-token";
    let output = redact_authorization(token);
    assert!(!output.contains("secret-token"));
}
```

Bài mở rộng: tạo log serializer chỉ cho phép field nằm trong allowlist; chạy grep trên artifact log test để bảo đảm không có `password`, `Authorization`, `MQTT_PASSWORD`, private key hoặc cookie.

---

# Phần D — Observability, MQTT reliability và deployment

## Bài 49 — Structured tracing cho async service

### Lời giải

```rust
use tracing::{info, instrument, warn};

#[instrument(skip(payload), fields(device_id = %device_id, message_id = %message_id))]
pub async fn process_message(
    device_id: &str,
    message_id: &str,
    payload: &[u8],
) -> Result<(), String> {
    info!(payload_bytes = payload.len(), "processing telemetry");
    if payload.is_empty() {
        warn!("empty payload");
        return Err("empty payload".into());
    }
    Ok(())
}
```

Trong async code, không giữ guard tạo bởi `span.enter()` qua `.await`; dùng `#[instrument]`, `span.in_scope` cho đoạn synchronous, hoặc `.instrument(span)` cho future. Tài liệu tracing cảnh báo guard qua await có thể tạo trace sai. [12]

Binary khởi tạo subscriber:

```rust
tracing_subscriber::fmt()
    .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
    .json()
    .init();
```

### Acceptance

Mỗi request có `request_id`; mỗi MQTT message có `message_id`; mọi retry log `attempt`, `delay`, `reason`; secret không xuất hiện.

---

## Bài 50 — MQTT broker bằng Docker Compose

### `docker-compose.yml`

```yaml
services:
  mqtt:
    image: eclipse-mosquitto:2
    ports:
      - "1883:1883"
    volumes:
      - ./ops/mosquitto.conf:/mosquitto/config/mosquitto.conf:ro
      - mqtt-data:/mosquitto/data
      - mqtt-log:/mosquitto/log
    healthcheck:
      test: ["CMD-SHELL", "mosquitto_sub -h 127.0.0.1 -t health -C 1 -W 2 || exit 1"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  mqtt-data:
  mqtt-log:
```

### `ops/mosquitto.conf` cho local development

```text
listener 1883 0.0.0.0
allow_anonymous true
persistence true
persistence_location /mosquitto/data/
log_dest stdout
```

`allow_anonymous true` chỉ dành cho local sandbox. Production phải dùng username/password hoặc certificate, ACL topic, TLS listener và không expose broker trực tiếp Internet nếu không có firewall/policy.

### Chạy

```bash
docker compose up -d mqtt
docker compose ps
docker compose logs -f mqtt
```

### Test publish/subscribe

```bash
mosquitto_sub -h 127.0.0.1 -t 'practice/#' -v
mosquitto_pub -h 127.0.0.1 -t 'practice/temperature' -m '{"value":21.5}'
```

Nếu máy không có Docker hoặc không được phép chạy daemon, dùng broker local cài trực tiếp hoặc broker test riêng; không coi Docker là bắt buộc cho phần code Rust.

---

## Bài 51 — MQTT reliability: deduplication, retry và outbox

### Database schema

```sql
CREATE TABLE processed_messages (
    message_id TEXT PRIMARY KEY,
    processed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE outbox (
    message_id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    payload BLOB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    delivered INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT NOT NULL
);
```

### Consumer algorithm

```text
1. Nhận MQTT message.
2. Parse và validate message_id.
3. Begin transaction.
4. INSERT message_id vào processed_messages.
5. Nếu conflict: message duplicate, commit/ignore.
6. Ghi domain state và outbox event trong cùng transaction.
7. Commit.
8. Worker đọc outbox, publish QoS phù hợp.
9. Chỉ đánh dấu delivered sau khi publish operation thành công theo policy.
```

Với QoS 1, delivery có thể lặp; business operation phải idempotent. MQTT PUBACK không đồng nghĩa với transaction database của ứng dụng đã hoàn tất. Outbox không giải quyết mọi distributed transaction, nhưng làm rõ boundary và tránh nhiều kiểu mất message.

### Test cases

| Case | Kết quả |
|---|---|
| message mới | ghi state một lần |
| duplicate message ID | không tăng counter lần hai |
| JSON sai | dead-letter/log, không crash loop |
| broker disconnect | outbox còn pending |
| publish retry | attempts tăng, có backoff |
| poison message | bị giới hạn retry, chuyển dead letter |

---

## Bài 52 — Dockerfile multi-stage cho Rust

```dockerfile
FROM rust:1.89-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY modules_lab ./modules_lab
COPY cargo_lab ./cargo_lab
COPY network_lab ./network_lab
RUN cargo build --release -p network-lab --bin network-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/network-server /usr/local/bin/network-server
ENV RUST_LOG=info
EXPOSE 3000
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/network-server"]
```

### Quy tắc build image

Không copy `.env`, private key, `target/` hoặc credential vào image. Dùng `.dockerignore`, multi-stage build, user non-root, CA certificates, health check và image scanning. Pin base image theo policy của tổ chức thay vì dùng tag không kiểm soát trong production.

### `.dockerignore`

```text
.git
.env
**/target
*.pem
*.key
```

---

## Bài 53 — Health, readiness và graceful shutdown

### Phân biệt

| Endpoint | Ý nghĩa |
|---|---|
| `/health/live` | Process còn sống, không nhất thiết dependency khỏe |
| `/health/ready` | Có thể nhận traffic; database/broker tối thiểu sẵn sàng |
| `/metrics` | Số liệu quan sát, phải bảo vệ nếu nhạy cảm |

Readiness không nên chạy query nặng mỗi request. Có thể cache trạng thái dependency với TTL ngắn. Liveness không nên fail chỉ vì broker tạm thời down nếu process vẫn có thể recover.

### Shutdown sequence

```text
1. Nhận SIGTERM/Ctrl+C.
2. Stop nhận request mới hoặc báo readiness false.
3. Ngừng nhận message mới.
4. Chờ task đang xử lý trong deadline.
5. Flush outbox/publish cần thiết.
6. Đóng database pool và broker client.
7. Exit.
```

Viết test shutdown bằng oneshot channel; assert task thoát trong deadline, không treo vì một `recv().await` vô hạn.

---

# Phần E — Integration, load test và capstone nghiệm thu

## Bài 54 — Integration test không phụ thuộc Internet

### Fixture server

Dùng Axum bind `127.0.0.1:0`, trả response JSON cố định, inject base URL vào client. Test các status 200, 400, 500, delay và malformed JSON. Không gọi `httpbin.org`, API thật hoặc broker công khai trong test CI.

### Test pattern

```rust
let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
let address = listener.local_addr()?;
let server = tokio::spawn(axum::serve(listener, app));

let client_result = call_api(&format!("http://{address}/data")).await;
assert!(client_result.is_ok());
server.abort();
```

Cần có cleanup ngay cả khi assertion fail; dùng guard hoặc test harness phù hợp nếu suite lớn.

---

## Bài 55 — Load test và saturation point

### Mục tiêu

Đo throughput và latency khi tăng concurrency. Không nhầm benchmark hàm thuần với load test service.

### Ma trận

| Concurrency | Requests | p50 | p95 | p99 | Error rate |
|---:|---:|---:|---:|---:|---:|
| 1 | 100 | ghi nhận | ghi nhận | ghi nhận | ghi nhận |
| 10 | 1.000 | ghi nhận | ghi nhận | ghi nhận | ghi nhận |
| 50 | 5.000 | ghi nhận | ghi nhận | ghi nhận | ghi nhận |
| 100 | 10.000 | ghi nhận | ghi nhận | ghi nhận | ghi nhận |

Tìm saturation point: nơi tăng concurrency không còn tăng throughput nhưng p95/p99 tăng mạnh hoặc error rate bắt đầu tăng. Đặt timeout và giới hạn connection; nếu không, load test chỉ đo khả năng tạo request của tool.

### Acceptance example

```text
p95 < 200 ms ở concurrency 50
error rate < 0.1%
không memory leak trong 10 phút
không có task bị treo sau shutdown
```

Các ngưỡng chỉ là ví dụ; phải thay bằng SLO của hệ thống thật.

---

## Bài 56 — Threat model cho Device Gateway

### Tài sản cần bảo vệ

Credential MQTT, JWT signing key, dữ liệu telemetry, tenant isolation, database và availability của broker/API.

### Đối tượng đe dọa

Client giả mạo, tenant khác đọc topic, replay message, payload quá lớn, HTTP SSRF, credential brute force, broker flood, log leak và dependency có lỗ hổng.

### Bảng kiểm soát

| Mối đe dọa | Kiểm soát |
|---|---|
| giả mạo client | TLS/mTLS, ACL, credential rotation |
| replay | message ID, timestamp/window, dedup store |
| payload quá lớn | giới hạn trước parse và giới hạn broker |
| cross-tenant | topic ACL + authorization ở application |
| secret leak | secret manager, redaction, non-root image |
| flood | rate limit, semaphore, broker quota |
| SQL injection | bind parameters, query review |
| JWT misuse | verify signature/exp/iss/aud/algorithm |
| shutdown data loss | outbox, drain, deadline |

Threat model phải gắn với test hoặc control cụ thể; không dừng ở danh sách rủi ro.

---

## Bài 57 — Capstone nghiệm thu cuối khóa

### Yêu cầu chức năng

Xây Device Gateway có HTTP REST, MQTT telemetry, WebSocket updates, SQLite persistence và JWT authentication. Domain service phải dùng chung cho mọi transport.

### Yêu cầu kỹ thuật

| Hạng mục | Điều kiện nghiệm thu |
|---|---|
| Module | `domain`, `service`, `http`, `mqtt`, `websocket`, `persistence`, `auth` tách rõ |
| API | `POST /auth/login`, `GET /devices/{id}`, `GET /health/live`, `GET /health/ready` |
| MQTT | subscribe telemetry, validate JSON, QoS 1, dedup message ID |
| Database | migration, transaction state + outbox, index theo device/time |
| Auth | Argon2 password hash, JWT exp/iss, role/tenant check |
| WebSocket | broadcast event, xử lý client chậm và close frame |
| Resilience | timeout, retry có điều kiện, backoff, bounded channel |
| Observability | request ID, message ID, structured tracing, error classification |
| Security | TLS plan, secret redaction, no plaintext credential, input limit |
| Testing | unit, integration, malformed input, duplicate, disconnect, timeout |
| Delivery | Dockerfile, compose local, README runbook, migration command |

### Definition of Done

Project chỉ được xem là hoàn tất khi:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo doc --workspace --no-deps
```

Ngoài compile/test, phải có test runtime local cho HTTP, TCP/UDP/WebSocket và broker integration tùy môi trường. MQTT test không được silently skip; nếu broker không có, test phải báo rõ là integration environment chưa sẵn sàng.

### Runbook tối thiểu

```text
1. Cài Rust stable, Docker và mosquitto CLI nếu cần.
2. Tạo .env từ .env.example; không thêm secret thật vào Git.
3. Chạy migration.
4. Khởi động MQTT broker local.
5. Khởi động service.
6. Kiểm tra /health/live và /health/ready.
7. Gửi telemetry test với message_id cố định.
8. Gửi lại cùng message_id và xác nhận không ghi trùng.
9. Ngắt broker, xác nhận outbox pending.
10. Khởi động broker lại, xác nhận worker retry.
11. Gửi SIGTERM, xác nhận graceful shutdown.
12. Chạy test suite và lưu artifact/log an toàn.
```

---

# Phần F — Playbook chẩn đoán chuyên sâu

## 1. Service không nhận MQTT message

Kiểm tra theo thứ tự: broker đang listen đúng host/port; client ID không bị đá bởi client khác; topic filter đúng; ACL cho phép subscribe; EventLoop có được poll liên tục; payload không bị reject ở validator; log có message ID; network/TLS có verify hostname.

## 2. API latency tăng nhưng CPU thấp

Khả năng thường gặp là upstream chậm, pool database cạn, semaphore quá nhỏ, lock contention, DNS/TLS handshake lặp vì client không được reuse, hoặc retry tạo thác request. Đo p50/p95/p99 theo từng span và kiểm tra pool wait time thay vì chỉ đo tổng request.

## 3. Memory tăng theo thời gian

Kiểm tra channel unbounded, broadcast receiver chậm, outbox không được drain, cache không có eviction, task spawn không join, read body không giới hạn và payload/frame length bị tin tưởng. Chụp heap/profile trong workload tái hiện, không suy đoán từ RSS một lần.

## 4. Test flaky

Loại bỏ sleep cố định; dùng notification/channel hoặc barrier. Dùng port ephemeral. Không dùng thời gian hệ thống thật nếu có clock abstraction. Không chia sẻ database file giữa test song song. Không phụ thuộc Internet, broker công khai hoặc thứ tự test.

## 5. Release build khác debug

Kiểm tra feature resolver, `cfg`, build script, environment, certificate store, native dependency, profile tối ưu và race timing. Chạy test với `--release` khi performance hoặc overflow phụ thuộc profile. Không dùng `debug_assert!` cho invariant security bắt buộc.

---

# Tài liệu tham khảo

[8]: https://docs.rs/sqlx/latest/sqlx/ "SQLx documentation"

[9]: https://docs.rs/argon2/latest/argon2/ "Argon2 password hashing documentation"

[10]: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/ "jsonwebtoken documentation"

[11]: https://docs.rs/criterion/latest/criterion/ "Criterion benchmark documentation"

[12]: https://docs.rs/tracing/latest/tracing/ "tracing structured diagnostics documentation"

[13]: https://docs.docker.com/compose/ "Docker Compose documentation"

[14]: https://mosquitto.org/man/mosquitto-conf-5.html "Mosquitto configuration manual"

[15]: https://doc.rust-lang.org/book/ "The Rust Programming Language"

**Tác giả:** Manus AI  
**Phạm vi:** Rust production engineering, persistence, auth, observability, MQTT reliability, Docker và capstone
