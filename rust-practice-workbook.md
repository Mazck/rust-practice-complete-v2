# Workbook thực hành Rust: Module, Cargo, Async và Networking

> **Mục tiêu:** hoàn thành một lộ trình thực hành có mã nguồn chạy được, đi từ module/path/visibility đến workspace, macro, build script, test, async, HTTP/REST, MQTT, TCP, UDP, WebSocket, TLS và project tích hợp.
>
> **Cách học:** với mỗi bài, hãy đọc mục “Yêu cầu”, tự viết trước trong thư mục `student/`, sau đó đối chiếu “Lời giải”. Các ví dụ mạng dùng loopback hoặc fixture local; MQTT yêu cầu broker local khi chạy thật.

## 0. Chuẩn bị môi trường

Cài Rust stable và kiểm tra:

```bash
rustc --version
cargo --version
rustup show active-toolchain
```

Tạo workspace thực hành:

```bash
mkdir rust-practice
cd rust-practice
cargo new --lib core_lib
cargo new --bin app
```

Sau khi hoàn thành 37 bài trong tài liệu này, học tiếp **[rust-production-workbook.md](rust-production-workbook.md)**. Phần nâng cao bổ sung correctness, property testing, benchmark, SQLx/SQLite, migration, transaction/outbox, Argon2id, JWT, authorization, rate limit, tracing, MQTT reliability, Docker Compose, health/readiness, load test, threat model và capstone nghiệm thu.

Các lệnh dùng thường xuyên:

| Mục đích | Lệnh |
|---|---|
| Kiểm tra nhanh | `cargo check` |
| Biên dịch mọi target | `cargo check --all-targets --all-features` |
| Chạy binary | `cargo run` |
| Chạy example | `cargo run --example ten_example` |
| Chạy test | `cargo test --all-targets --all-features` |
| Chạy doctest | `cargo test --doc` |
| Format | `cargo fmt --all -- --check` |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` |
| Xem dependency | `cargo tree -e features` |
| Xem metadata | `cargo metadata --no-deps --format-version 1` |
| Sinh tài liệu | `cargo doc --no-deps --open` |

---

# Phần I — Module, path và visibility

## Bài 1 — Tạo cây module và gọi hàm bằng path

### Mục tiêu

Luyện `mod`, module inline, path tương đối và path tuyệt đối.

### Yêu cầu

Tạo binary có module `math` chứa các hàm `add`, `sub`, `mul`. Hàm `add` gọi được từ `main`; hàm helper nội bộ không được gọi trực tiếp từ ngoài module.

### Lời giải

```rust
// src/main.rs
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        validate(a, b);
        a + b
    }

    pub fn sub(a: i32, b: i32) -> i32 {
        a - b
    }

    fn validate(_a: i32, _b: i32) {
        // Chi tiết nội bộ; không cần public.
    }
}

fn main() {
    let sum = crate::math::add(2, 3);
    let difference = math::sub(8, 5);
    println!("sum={sum}, difference={difference}");
}
```

### Chạy và kết quả

```bash
cargo run
# sum=5, difference=3
```

`crate::math::add` là absolute path từ crate root. `math::sub` là relative path từ scope hiện tại. `validate` private nên `main` không thể gọi `math::validate()`.

### Mở rộng

Thêm `pub(crate) fn debug_state()` và thử gọi từ module khác trong cùng crate. Sau đó thử gọi từ integration test để quan sát compiler từ chối.

---

## Bài 2 — Tách module thành file và thư mục

### Mục tiêu

Hiểu quan hệ giữa `mod.rs`, file module và module con.

### Cấu trúc yêu cầu

```text
src/
├── main.rs
└── domain/
    ├── mod.rs
    ├── user.rs
    └── role.rs
```

### Lời giải

```rust
// src/main.rs
mod domain;

fn main() {
    let user = domain::User::new("An", domain::Role::Admin);
    println!("{} ({:?})", user.name(), user.role());
}
```

```rust
// src/domain/mod.rs
mod role;
mod user;

pub use role::Role;
pub use user::User;
```

```rust
// src/domain/role.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
}
```

```rust
// src/domain/user.rs
use super::Role;

pub struct User {
    name: String,
    role: Role,
}

impl User {
    pub fn new(name: impl Into<String>, role: Role) -> Self {
        Self { name: name.into(), role }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}
```

`role` và `user` là module con private của `domain`, nhưng `pub use` tạo facade nên code bên ngoài chỉ cần `domain::User` và `domain::Role`.

### Kiểm tra lỗi

Nếu xóa `pub use role::Role`, `domain::Role` sẽ không tồn tại dù `Role` trong `role.rs` là `pub`. Visibility của item và khả năng đi qua từng module trên path đều phải đúng.

---

## Bài 3 — So sánh `self`, `super`, `crate` và `Self`

### Yêu cầu

Tạo `src/service/mod.rs` với module `repository`. Trong `service::run`, gọi hằng số bằng `self::`, gọi repository bằng `super::repository`, gọi config ở crate root bằng `crate::CONFIG`, và gọi associated function bằng `Self`.

### Lời giải

```rust
// src/main.rs
const CONFIG: &str = "development";

mod service {
    pub mod repository {
        pub fn load() -> &'static str {
            "data"
        }
    }

    pub struct Service;

    impl Service {
        pub fn new() -> Self {
            Self
        }

        pub fn run(&self) -> String {
            let _local = self::repository::load();
            let _parent = super::repository::load();
            let config = crate::CONFIG;
            format!("{}:{config}", Self::label())
        }

        fn label() -> &'static str {
            "service"
        }
    }
}

fn main() {
    println!("{}", service::Service::new().run());
}
```

Trong method thuộc `impl Service`, `Self` là kiểu `Service`; trong path module, `self` là module hiện tại. Hai khái niệm này có chữ giống nhau nhưng ngữ cảnh khác nhau.

---

## Bài 4 — Visibility đầy đủ: `pub`, `pub(crate)`, `pub(super)`, `pub(in ...)`

### Yêu cầu

Tạo ba tầng `api::internal::parser`. Cho phép `parser` dùng helper của `api` nhưng không công khai helper đó cho crate khác.

### Lời giải

```rust
mod api {
    pub(crate) fn crate_only() -> &'static str {
        "inside crate"
    }

    pub mod internal {
        pub(super) fn parent_only() -> &'static str {
            "visible to api"
        }

        pub mod parser {
            pub(in crate::api) fn api_only() -> &'static str {
                "visible within api"
            }

            pub fn parse() -> String {
                format!("{} / {} / {}", crate_only(), super::parent_only(), api_only())
            }
        }
    }
}

fn main() {
    println!("{}", api::internal::parser::parse());
}
```

`pub(crate)` mở cho mọi module trong cùng crate. `pub(super)` mở cho module cha trực tiếp. `pub(in crate::api)` mở trong subtree `crate::api`. `pub` mở theo boundary crate nếu mọi parent module trên path cũng công khai.

---

## Bài 5 — Re-export facade cho library crate

### Mục tiêu

Thiết kế public API ổn định, che implementation detail.

### Lời giải

```rust
// src/lib.rs
mod implementation;

pub use implementation::{Email, User, UserBuilder};

mod implementation {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Email(String);

    impl Email {
        pub fn parse(value: impl Into<String>) -> Result<Self, &'static str> {
            let value = value.into();
            if value.contains('@') {
                Ok(Self(value))
            } else {
                Err("invalid email")
            }
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct User {
        pub name: String,
        pub email: Email,
    }

    pub struct UserBuilder {
        name: String,
        email: Option<Email>,
    }

    impl UserBuilder {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into(), email: None }
        }

        pub fn email(mut self, value: impl Into<String>) -> Result<Self, &'static str> {
            self.email = Some(Email::parse(value)?);
            Ok(self)
        }

        pub fn build(self) -> Result<User, &'static str> {
            Ok(User {
                name: self.name,
                email: self.email.ok_or("email is required")?,
            })
        }
    }
}
```

Client bên ngoài chỉ dùng:

```rust
use my_library::{User, UserBuilder};

let user = UserBuilder::new("An")
    .email("an@example.com")?
    .build()?;
```

Tên module `implementation` có thể đổi mà không bắt client đổi import, miễn facade giữ nguyên.

---

## Bài 6 — Module sibling, `super` và import kiểu dữ liệu

### Yêu cầu

Module `user` dùng enum `Role` ở sibling `role`. Không truy cập sibling bằng `role::Role` trực tiếp nếu scope không có import; hãy dùng `super::Role` sau re-export.

### Lời giải

```rust
mod domain {
    mod role {
        #[derive(Debug, Clone, Copy)]
        pub enum Role { User, Admin }
    }

    pub use role::Role;

    mod user {
        use super::Role;

        pub struct User {
            pub name: String,
            pub role: Role,
        }

        impl User {
            pub fn new(name: impl Into<String>, role: Role) -> Self {
                Self { name: name.into(), role }
            }
        }
    }

    pub use user::User;
}

fn main() {
    let user = domain::User::new("An", domain::Role::Admin);
    println!("{} {:?}", user.name, user.role);
}
```

Đây là pattern barrel module: module cha gom và tái xuất API của module con.

---

## Bài 7 — Library và binary trong cùng package

### Yêu cầu

`src/lib.rs` cung cấp `greet`; `src/main.rs` gọi library bằng tên crate. Đổi tên package có dấu gạch ngang và library name có dấu gạch dưới.

### Lời giải

```toml
[package]
name = "hello-service"
version = "0.1.0"
edition = "2024"

[lib]
name = "hello_service"
path = "src/lib.rs"
```

```rust
// src/lib.rs
pub fn greet(name: &str) -> String {
    format!("Hello, {name}")
}
```

```rust
// src/main.rs
fn main() {
    println!("{}", hello_service::greet("Rust"));
}
```

Trong code Rust, crate name dùng dấu gạch dưới; package name trên crates.io có thể dùng dấu gạch ngang. `crate::` trong `main.rs` trỏ tới binary crate hiện tại, không trỏ sang library crate cùng package.

---

## Bài 8 — Integration test chỉ dùng public API

### Yêu cầu

Tạo `tests/public_api.rs`, gọi facade public của library và chứng minh module private không thể truy cập.

### Lời giải

```rust
// src/lib.rs
mod internal {
    pub fn public_inside_private_module() -> &'static str {
        "ok"
    }
}

pub fn api() -> &'static str {
    internal::public_inside_private_module()
}
```

```rust
// tests/public_api.rs
#[test]
fn uses_public_facade() {
    assert_eq!(my_library::api(), "ok");
}
```

Dòng sau phải lỗi nếu bỏ comment:

```rust
// my_library::internal::public_inside_private_module();
```

Integration test là crate bên ngoài library crate, vì vậy nó mô phỏng đúng boundary của người dùng dependency.

---

## Bài 9 — Xử lý lỗi module bằng `thiserror`

### Lời giải

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("name is empty")]
    EmptyName,
    #[error("invalid email: {0}")]
    InvalidEmail(String),
}

pub fn create_user(name: &str, email: &str) -> Result<(), DomainError> {
    if name.trim().is_empty() {
        return Err(DomainError::EmptyName);
    }
    if !email.contains('@') {
        return Err(DomainError::InvalidEmail(email.to_owned()));
    }
    Ok(())
}
```

Bài tập mở rộng là tách `error.rs`, re-export `DomainError`, rồi viết unit test cho từng variant. Không dùng `unwrap()` ở public API nếu input có thể sai.

---

## Bài 10 — Workspace hai package

### Cấu trúc

```text
workspace/
├── Cargo.toml
├── domain/
│   ├── Cargo.toml
│   └── src/lib.rs
└── server/
    ├── Cargo.toml
    └── src/main.rs
```

### Lời giải

```toml
# Cargo.toml root
[workspace]
members = ["domain", "server"]
resolver = "3"
```

```toml
# server/Cargo.toml
[package]
name = "server"
version = "0.1.0"
edition = "2024"

[dependencies]
domain = { path = "../domain" }
```

```rust
// domain/src/lib.rs
pub struct DeviceId(pub String);
```

```rust
// server/src/main.rs
use domain::DeviceId;

fn main() {
    let id = DeviceId("device-01".to_owned());
    println!("{}", id.0);
}
```

Chạy từ root:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p server
```

---

# Phần II — Cargo, feature, macro, build script và test

## Bài 11 — Cargo target: bin, example, test

### Yêu cầu

Tạo một package có library, binary, example và integration test. Mỗi target gọi cùng facade library.

### Lời giải

```text
src/lib.rs
src/main.rs
examples/inspect.rs
tests/public_api.rs
```

```rust
// src/lib.rs
pub fn answer() -> u32 { 42 }
```

```rust
// src/main.rs
fn main() { println!("{}", target_demo::answer()); }
```

```rust
// examples/inspect.rs
fn main() { println!("example={}", target_demo::answer()); }
```

```rust
// tests/public_api.rs
#[test]
fn answer_is_stable() { assert_eq!(target_demo::answer(), 42); }
```

```bash
cargo run
cargo run --example inspect
cargo test
```

---

## Bài 12 — Feature flag và `cfg`

### Manifest

```toml
[features]
default = ["pretty"]
pretty = []
compact = []
```

### Code

```rust
pub fn render(value: &str) -> String {
    #[cfg(feature = "pretty")]
    {
        return format!("*** {value} ***");
    }

    #[cfg(all(not(feature = "pretty"), feature = "compact"))]
    {
        return value.to_owned();
    }

    #[cfg(not(any(feature = "pretty", feature = "compact")))]
    {
        return format!("[{value}]");
    }
}
```

### Chạy

```bash
cargo run
cargo run --no-default-features --features compact
cargo check --all-features
```

Không viết feature theo kiểu “feature này tắt feature kia” nếu không thật sự cần; Cargo features có tính additive và có thể được bật bởi nhiều dependency trong graph. Thiết kế feature nên tránh trạng thái không hợp lệ bằng compile-time check hoặc API rõ ràng. [1] [2]

---

## Bài 13 — `macro_rules!` tạo hàm log

### Yêu cầu

Viết macro `log_value!` nhận tên và expression, in ra tên cùng giá trị. Macro phải hỗ trợ nhiều cặp.

### Lời giải

```rust
macro_rules! log_value {
    ($name:ident = $value:expr) => {
        println!("{} = {:?}", stringify!($name), $value);
    };
    ($($name:ident = $value:expr),+ $(,)?) => {
        $(log_value!($name = $value);)+
    };
}

fn main() {
    let count = 3;
    let name = "Rust";
    log_value!(count = count, name = name);
}
```

`stringify!` lấy token thành chuỗi compile-time; `$value:expr` cho phép expression; repetition `$(...)+` lặp một hoặc nhiều lần.

---

## Bài 14 — Macro export và `$crate`

### Lời giải

```rust
#[doc(hidden)]
pub fn __format_internal(value: &str) -> String {
    format!("<{}>", value)
}

#[macro_export]
macro_rules! format_public {
    ($value:expr) => {
        $crate::__format_internal($value)
    };
}
```

`$crate` trỏ về crate định nghĩa macro, không phụ thuộc tên mà dependency được import ở crate gọi. Đây là cách tránh lỗi khi crate được rename trong `Cargo.toml`.

---

## Bài 15 — Build script sinh module

### `Cargo.toml`

```toml
[build-dependencies]
```

### `build.rs`

```rust
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=APP_VERSION");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let version = env::var("APP_VERSION").unwrap_or_else(|_| "dev".to_owned());
    let code = format!(
        "/// Version generated by build.rs.\npub const APP_VERSION: &str = {version:?};\n"
    );
    fs::write(out_dir.join("generated.rs"), code).unwrap();
}
```

### `src/lib.rs`

```rust
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

### Chạy

```bash
APP_VERSION=1.2.3 cargo run
cargo clean
```

`OUT_DIR` là thư mục build dành cho artifact sinh tự động. `rerun-if-*` giúp Cargo biết khi nào cần chạy lại build script. Không ghi file sinh vào `src/` nếu file đó chỉ thuộc build output. [3]

---

## Bài 16 — Unit test, integration test và doctest

### Lời giải

```rust
/// Cộng hai số.
///
/// # Examples
///
/// ```
/// assert_eq!(practice::add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_numbers() {
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    #[should_panic(expected = "invalid")]
    fn validates_input() {
        panic!("invalid input");
    }
}
```

Chạy:

```bash
cargo test
cargo test --doc
cargo test adds_numbers
```

Unit test có thể truy cập private item trong module test; integration test ở `tests/` chỉ dùng public API. Doctest vừa kiểm tra tài liệu vừa kiểm tra khả năng compile của ví dụ.

---

## Bài 17 — Clippy và API quality

### Yêu cầu

Viết code có `unwrap`, clone thừa và vòng lặp thủ công, sau đó sửa theo Clippy.

### Lời giải tốt hơn

```rust
pub fn parse_positive(value: &str) -> Result<u32, std::num::ParseIntError> {
    value.parse::<u32>()
}

pub fn doubled(values: &[u32]) -> Vec<u32> {
    values.iter().map(|value| value * 2).collect()
}
```

Chạy:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

# Phần III — Async và concurrency

## Bài 18 — Tokio task và channel

### Lời giải

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<u32>(8);

    let worker = tokio::spawn(async move {
        let mut total = 0;
        while let Some(value) = rx.recv().await {
            total += value;
        }
        total
    });

    for value in 1..=5 {
        tx.send(value).await?;
    }
    drop(tx);

    println!("total={}", worker.await?);
    Ok(())
}
```

Điểm quan trọng là `drop(tx)` để receiver nhận `None` và kết thúc. Channel bounded tạo backpressure: sender sẽ chờ nếu buffer đầy.

---

## Bài 19 — `select!`, timeout và shutdown

### Lời giải

```rust
use std::time::Duration;
use tokio::{signal, time::{sleep, timeout}};

#[tokio::main]
async fn main() {
    tokio::select! {
        result = timeout(Duration::from_secs(2), slow_operation()) => {
            match result {
                Ok(value) => println!("done: {value}"),
                Err(_) => println!("timed out"),
            }
        }
        result = signal::ctrl_c() => {
            println!("shutdown: {result:?}");
        }
    }
}

async fn slow_operation() -> &'static str {
    sleep(Duration::from_secs(1)).await;
    "finished"
}
```

Bài mở rộng: dùng một `broadcast::Sender<()>` để báo shutdown đồng thời cho HTTP server, MQTT loop và WebSocket tasks.

---

## Bài 20 — Giới hạn concurrency bằng Semaphore

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limit = Arc::new(Semaphore::new(2));
    let mut tasks = Vec::new();

    for id in 0..6 {
        let limit = Arc::clone(&limit);
        tasks.push(tokio::spawn(async move {
            let _permit = limit.acquire_owned().await.unwrap();
            println!("start {id}");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!("end {id}");
        }));
    }

    for task in tasks { task.await?; }
    Ok(())
}
```

Quan sát log để thấy tối đa hai task chạy trong vùng được bảo vệ.

---

# Phần IV — HTTP client và REST API

## Bài 21 — Gọi API JSON typed

### Manifest

```toml
reqwest = { version = "0.13", default-features = false, features = ["json", "query", "rustls"] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Lời giải

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct User { id: u64, name: String }

#[derive(Debug, Serialize)]
struct NewUser<'a> { name: &'a str }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .https_only(true)
        .build()?;

    let user: User = client
        .get("https://api.example.com/users/1")
        .bearer_auth(std::env::var("API_TOKEN")?)
        .send().await?
        .error_for_status()?
        .json().await?;

    println!("{} {}", user.id, user.name);

    let _: serde_json::Value = client
        .post("https://api.example.com/users")
        .json(&NewUser { name: "An" })
        .send().await?
        .error_for_status()?
        .json().await?;
    Ok(())
}
```

Không chạy endpoint giả này nếu chưa có API thật. Khi thực hành local, dùng fixture Axum trong project đi kèm.

---

## Bài 22 — Retry API an toàn

### Yêu cầu

Retry GET tối đa 4 lần, exponential backoff, chỉ retry lỗi transport và status 5xx/429. Không retry 400/401/403.

### Lời giải

```rust
use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

async fn get_retry(client: &Client, url: &str) -> Result<Response, reqwest::Error> {
    let mut delay = Duration::from_millis(100);

    for attempt in 0..4 {
        match client.get(url).send().await {
            Ok(response)
                if response.status().is_success() => return Ok(response),
            Ok(response)
                if (response.status().is_server_error()
                    || response.status() == StatusCode::TOO_MANY_REQUESTS)
                    && attempt < 3 => {}
            Ok(response) => return response.error_for_status(),
            Err(error) if attempt < 3 => eprintln!("retry {attempt}: {error}"),
            Err(error) => return Err(error),
        }
        sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(3));
    }
    unreachable!()
}
```

Bài mở rộng: thêm jitter, đọc `Retry-After`, tổng deadline và idempotency key cho POST.

---

## Bài 23 — Axum route, JSON body và State

### Lời giải

```rust
use axum::{extract::{Path, State}, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState { users: Arc<Mutex<Vec<User>>> }

#[derive(Clone, Serialize, Deserialize)]
struct User { id: u64, name: String }

async fn list(State(state): State<AppState>) -> Json<Vec<User>> {
    Json(state.users.lock().unwrap().clone())
}

async fn get_one(Path(id): Path<u64>, State(state): State<AppState>) -> Json<Option<User>> {
    Json(state.users.lock().unwrap().iter().find(|u| u.id == id).cloned())
}

async fn create(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Json<User> {
    state.users.lock().unwrap().push(user.clone());
    Json(user)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/users", get(list).post(create))
        .route("/users/{id}", get(get_one))
        .with_state(state)
}
```

Trong production, không giữ lock qua `.await`; thay `std::sync::Mutex` bằng kho dữ liệu hoặc `tokio::sync::Mutex` phù hợp. Handler nên trả error type chuyển được thành status code.

---

# Phần V — TCP, UDP và framing

## Bài 24 — TCP echo server/client

### Server

```rust
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move { handle(stream).await.unwrap(); });
    }
}

async fn handle(stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        writer.write_all(format!("echo: {line}\n").as_bytes()).await?;
    }
    Ok(())
}
```

### Client

```rust
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:7000").await?;
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    writer.write_all(b"one\ntwo\n").await?;
    writer.shutdown().await?;
    while let Some(line) = lines.next_line().await? { println!("{line}"); }
    Ok(())
}
```

TCP là byte stream; newline ở đây là protocol framing do ứng dụng tự định nghĩa.

---

## Bài 25 — TCP length-prefix protocol

```rust
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, data: &[u8]) -> std::io::Result<()> {
    let len = u32::try_from(data.len())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "too large"))?;
    w.write_u32(len).await?;
    w.write_all(data).await
}

async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let len = r.read_u32().await?;
    if len > 1024 * 1024 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "too large"));
    }
    let mut data = vec![0; len as usize];
    r.read_exact(&mut data).await?;
    Ok(data)
}
```

Bài mở rộng: serialize payload bằng `serde_json::to_vec`, thêm version byte, request ID và checksum.

---

## Bài 26 — UDP echo có timeout

```rust
use std::time::Duration;
use tokio::{net::UdpSocket, time::timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = UdpSocket::bind("127.0.0.1:9000").await?;
    let client = UdpSocket::bind("127.0.0.1:0").await?;
    client.send_to(b"ping", "127.0.0.1:9000").await?;

    let mut buf = [0; 2048];
    let (size, peer) = timeout(Duration::from_secs(2), server.recv_from(&mut buf)).await??;
    server.send_to(&buf[..size], peer).await?;
    Ok(())
}
```

UDP không đảm bảo delivery/order/deduplication. Hãy viết sequence number và ACK nếu bài toán cần reliability.

---

# Phần VI — WebSocket và MQTT

## Bài 27 — WebSocket echo

### Server

```rust
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9100").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut ws = accept_async(stream).await.unwrap();
            while let Some(message) = ws.next().await {
                let message = message.unwrap();
                if message.is_close() { break; }
                ws.send(message).await.unwrap();
            }
        });
    }
}
```

### Client

```rust
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut ws, response) = connect_async("ws://127.0.0.1:9100").await?;
    println!("{}", response.status());
    ws.send(Message::Text("hello".into())).await?;
    if let Some(message) = ws.next().await { println!("{:?}", message?); }
    Ok(())
}
```

Bài mở rộng: tách `Sink`/`Stream`, dùng `mpsc` để gửi message từ nhiều producer, giới hạn message size và broadcast trạng thái.

---

## Bài 28 — MQTT publish/subscribe bằng rumqttc

### Manifest

```toml
rumqttc = "0.25"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Lời giải

```rust
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = std::env::var("MQTT_PORT").unwrap_or_else(|_| "1883".to_owned()).parse()?;
    let mut options = MqttOptions::new("practice-client", host, port);
    options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(options, 32);
    client.subscribe("practice/temperature", QoS::AtLeastOnce).await?;
    client.publish(
        "practice/temperature",
        QoS::AtLeastOnce,
        false,
        br#"{"value": 25.5, "unit": "C", "message_id": "m-1"}"#,
    ).await?;

    for _ in 0..5 {
        match eventloop.poll().await? {
            Event::Incoming(Packet::Publish(message)) => {
                println!("{} {:?}", message.topic, message.payload);
            }
            event => println!("{event:?}"),
        }
    }
    Ok(())
}
```

Chạy broker local trước. Với Mosquitto mặc định:

```bash
mosquitto -p 1883
cargo run --example mqtt_client
```

`EventLoop` phải liên tục được poll; nếu chặn hoặc bỏ poll, connection không tiến triển. QoS 1 có thể giao trùng, vì vậy payload nghiệp vụ nên có `message_id` và consumer phải idempotent. [4]

### Bài mở rộng MQTT

1. Tách event loop thành task riêng và dùng shutdown channel.
2. Parse payload bằng `serde_json::from_slice`.
3. Dùng topic `tenant/{tenant}/device/{device}/telemetry/{metric}`.
4. Dedupe `message_id` bằng `HashSet` có giới hạn.
5. Thêm TLS và CA nội bộ; tuyệt đối không tắt certificate verification.

---

# Phần VII — TLS, bảo mật và độ tin cậy

## Bài 29 — HTTPS bắt buộc TLS

```rust
let client = reqwest::Client::builder()
    .https_only(true)
    .timeout(std::time::Duration::from_secs(10))
    .build()?;

let response = client.get("https://example.com").send().await?;
println!("{}", response.status());
```

Không dùng `danger_accept_invalid_certs(true)` trong production. Nếu có CA private, thêm đúng certificate/root store. Rustls cần hostname để đối chiếu certificate; không dùng IP nếu certificate không chứa IP đó. [5]

---

## Bài 30 — Xử lý lỗi theo tầng

### Yêu cầu

Thiết kế error enum phân biệt config, timeout, transport, HTTP status, decode và protocol.

### Lời giải

```rust
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("configuration: {0}")]
    Config(String),
    #[error("timeout")]
    Timeout,
    #[error("transport: {0}")]
    Transport(String),
    #[error("HTTP status {0}")]
    Http(u16),
    #[error("decode: {0}")]
    Decode(String),
    #[error("protocol: {0}")]
    Protocol(String),
}
```

Trong Axum, map error nội bộ thành `(StatusCode, Json<PublicError>)`; không trả token, stack trace hoặc nội dung certificate cho client.

---

## Bài 31 — Structured logging và redaction

```rust
use tracing::{info, instrument};

#[instrument(skip(token), fields(endpoint = %url))]
async fn call_api(url: &str, token: &str) -> Result<(), reqwest::Error> {
    info!(token_present = !token.is_empty(), "calling upstream");
    Ok(())
}
```

Không log `token` trực tiếp. Thêm `tracing-subscriber` và dùng `RUST_LOG=info`. Bài mở rộng: viết helper redaction cho `Authorization`, `Cookie`, MQTT password và private payload.

---

## Bài 32 — Test mạng bằng port ephemeral

### Lời giải

```rust
#[tokio::test]
async fn binds_ephemeral_port() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    assert_ne!(address.port(), 0);
    Ok(())
}
```

Dùng port `0` tránh xung đột giữa test chạy song song. Test protocol bằng loopback hoặc fixture, không phụ thuộc API Internet trong unit test.

---

# Phần VIII — Project tích hợp cuối khóa

## Bài 33 — Device Gateway: HTTP + MQTT + WebSocket + domain service

### Mục tiêu

Xây một gateway có ba transport:

```text
MQTT telemetry --> parser --> DeviceService --> in-memory state
HTTP GET state  <----------- DeviceService
WebSocket      <------------ broadcast updates
```

### Cấu trúc

```text
device-gateway/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── domain.rs
│   ├── service.rs
│   ├── http.rs
│   ├── mqtt.rs
│   └── websocket.rs
└── tests/
    ├── domain.rs
    └── http.rs
```

### Manifest

```toml
[package]
name = "device-gateway"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
futures-util = "0.3"
rumqttc = "0.25"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.30"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

### Domain model

```rust
// src/domain.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Telemetry {
    pub device_id: String,
    pub value: f32,
    pub unit: String,
    pub message_id: String,
}

impl Telemetry {
    pub fn validate(&self) -> Result<(), String> {
        if self.device_id.trim().is_empty() { return Err("device_id empty".into()); }
        if self.message_id.trim().is_empty() { return Err("message_id empty".into()); }
        if !self.value.is_finite() { return Err("value not finite".into()); }
        Ok(())
    }
}
```

### Service state

```rust
// src/service.rs
use crate::domain::Telemetry;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct DeviceService {
    state: Arc<RwLock<HashMap<String, Telemetry>>>,
    updates: broadcast::Sender<Telemetry>,
}

impl DeviceService {
    pub fn new() -> Self {
        let (updates, _) = broadcast::channel(128);
        Self { state: Arc::new(RwLock::new(HashMap::new())), updates }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Telemetry> {
        self.updates.subscribe()
    }

    pub async fn record(&self, telemetry: Telemetry) -> Result<(), String> {
        telemetry.validate()?;
        self.state.write().await.insert(telemetry.device_id.clone(), telemetry.clone());
        let _ = self.updates.send(telemetry);
        Ok(())
    }

    pub async fn get(&self, device_id: &str) -> Option<Telemetry> {
        self.state.read().await.get(device_id).cloned()
    }
}
```

### HTTP module

```rust
// src/http.rs
use crate::service::DeviceService;
use axum::{extract::{Path, State}, http::StatusCode, response::Json, routing::get, Router};
use std::sync::Arc;

pub fn router(service: DeviceService) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/devices/{id}/telemetry", get(telemetry))
        .with_state(Arc::new(service))
}

async fn health() -> &'static str { "ok" }

async fn telemetry(
    State(service): State<Arc<DeviceService>>,
    Path(id): Path<String>,
) -> Result<Json<crate::domain::Telemetry>, StatusCode> {
    service.get(&id).await.map(Json).ok_or(StatusCode::NOT_FOUND)
}
```

### MQTT module

```rust
// src/mqtt.rs
use crate::{domain::Telemetry, service::DeviceService};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

pub async fn run(service: DeviceService) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("MQTT_PORT").unwrap_or_else(|_| "1883".into()).parse()?;
    let mut options = MqttOptions::new("device-gateway", host, port);
    options.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(options, 64);
    client.subscribe("devices/+/telemetry", QoS::AtLeastOnce).await?;

    loop {
        match eventloop.poll().await? {
            Event::Incoming(Packet::Publish(message)) => {
                match serde_json::from_slice::<Telemetry>(&message.payload) {
                    Ok(value) => {
                        if let Err(error) = service.record(value).await {
                            eprintln!("invalid telemetry: {error}");
                        }
                    }
                    Err(error) => eprintln!("invalid JSON: {error}"),
                }
            }
            _ => {}
        }
    }
}
```

### Main assembly

```rust
// src/main.rs
mod domain;
mod http;
mod mqtt;
mod service;

use service::DeviceService;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let service = DeviceService::new();
    let http_service = service.clone();
    let mqtt_service = service.clone();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let app = http::router(http_service);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    });
    tasks.spawn(async move { mqtt::run(mqtt_service).await.map_err(|e| e.to_string()) });

    tokio::select! {
        result = tasks.join_next() => println!("task ended: {result:?}"),
        result = tokio::signal::ctrl_c() => println!("shutdown: {result:?}"),
    }
    Ok(())
}
```

Đây là skeleton học tập; production cần thêm graceful shutdown cho MQTT event loop, message deduplication, authentication, TLS, bounded input, metrics, persistence và health/readiness riêng.

### Checklist hoàn thành project

| Hạng mục | Đã hoàn thành khi |
|---|---|
| Module | Mỗi transport nằm trong module riêng, public facade nhỏ |
| Domain | `Telemetry::validate` không phụ thuộc network |
| HTTP | `/health` và `/devices/{id}/telemetry` có test |
| MQTT | EventLoop liên tục poll, JSON được validate |
| WebSocket | Broadcast update, xử lý client chậm |
| Error | Không trả lỗi nội bộ nhạy cảm |
| Async | Không block executor, có timeout/shutdown |
| Security | TLS, auth, ACL, secret injection |
| Test | Unit, integration, fixture và malformed input |
| Operations | Log, metrics, readiness, graceful shutdown |

---

# Phần IX — Bài kiểm tra tổng hợp và đáp án định hướng

## Bài 34 — Chẩn đoán lỗi module

### Mã lỗi

```rust
mod parent {
    mod child {
        pub fn run() {}
    }
}

fn main() {
    parent::child::run();
}
```

### Câu hỏi

Vì sao lỗi? Sửa tối thiểu nhưng vẫn giữ `child::run` public.

### Đáp án

`child` là private nên path không đi qua được. Sửa:

```rust
mod parent {
    pub mod child {
        pub fn run() {}
    }
}
```

Nếu muốn giấu tên `child`, dùng facade:

```rust
mod parent {
    mod child { pub fn run() {} }
    pub fn run() { child::run(); }
}
```

---

## Bài 35 — Chẩn đoán lỗi `crate::` giữa binary và library

### Mã lỗi

```rust
// src/main.rs
crate::api::router();
```

### Đáp án

Nếu `api` nằm trong `src/lib.rs`, `main.rs` phải gọi library bằng tên crate:

```rust
my_package::api::router();
```

`crate::` trong `main.rs` trỏ binary crate; `crate::` trong `lib.rs` trỏ library crate. Một package có thể chứa nhiều crate nên hai crate root không dùng chung `crate::` namespace.

---

## Bài 36 — Chẩn đoán future không `Send`

### Mã lỗi

```rust
let guard = std::sync::Mutex::new(0).lock().unwrap();
tokio::spawn(async move {
    println!("{guard}");
});
```

### Hướng sửa

Không đưa guard borrow vào task. Dùng `Arc<Mutex<T>>`, lock bên trong task và thả guard trước `.await`:

```rust
let state = std::sync::Arc::new(tokio::sync::Mutex::new(0));
let task_state = state.clone();
tokio::spawn(async move {
    let value = *task_state.lock().await;
    println!("{value}");
});
```

---

## Bài 37 — Chọn protocol đúng

| Tình huống | Đáp án nên chọn | Lý do |
|---|---|---|
| Gọi API thanh toán | HTTPS/Reqwest | Request/response, TLS, JSON |
| Cảm biến cần publish telemetry | MQTT | Broker, topic, QoS, reconnect |
| Chat browser realtime | WebSocket | Hai chiều, message stream |
| File protocol riêng tin cậy | TCP + framing | Ordered byte stream |
| Discovery nhanh trong LAN | UDP | Datagram, chấp nhận loss hoặc tự ACK |
| API server nhiều route | Axum | Router, extractor, middleware |
| CPU-heavy transform | Rayon/worker | Tokio ưu tiên I/O-bound |

---

# Phần X — Tiêu chí chấm bài

| Mức | Tiêu chí |
|---|---|
| Đạt | Code compile, chạy happy path, module/path đúng |
| Khá | Có typed error, test, không `unwrap` trong boundary, có timeout |
| Tốt | Có backpressure, retry có điều kiện, structured logging, fixture local |
| Giỏi | Có graceful shutdown, TLS đúng, auth/ACL, idempotency, property/fuzz test |
| Production-ready | Có observability, dependency audit, migration/rotation, load test, threat model và runbook |

Một bài chỉ “chạy được” chưa đồng nghĩa production-ready. Mạng luôn có disconnect, delay, duplicate, malformed input và credential failure; bài làm tốt phải mô phỏng các trường hợp đó.

---

# Tài liệu tham khảo

[1]: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html "The Rust Programming Language — Defining Modules to Control Scope and Privacy"

[2]: https://doc.rust-lang.org/cargo/reference/features.html "The Cargo Book — Features"

[3]: https://doc.rust-lang.org/cargo/reference/build-scripts.html "The Cargo Book — Build Scripts"

[4]: https://docs.rs/rumqttc/latest/rumqttc/ "rumqttc documentation"

[5]: https://docs.rs/rustls/latest/rustls/ "rustls documentation"

[6]: https://docs.rs/reqwest/latest/reqwest/ "reqwest documentation"

[7]: https://docs.rs/axum/latest/axum/ "axum documentation"

[8]: https://docs.rs/tokio/latest/tokio/ "Tokio documentation"

[9]: https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/ "tokio-tungstenite documentation"

[10]: https://docs.rs/serde_json/latest/serde_json/ "serde_json documentation"

**Tác giả:** Manus AI  
**Ngôn ngữ:** Tiếng Việt  
**Phiên bản:** Bộ bài tập thực hành mở rộng
