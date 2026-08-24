# Rust Module, Crate, Package và lập trình mạng trong Rust

> **Mục tiêu của tài liệu:** giải thích từ gốc đến ngọn cách Rust tổ chức mã nguồn và cách gọi một hàm, kiểu dữ liệu, hằng số, trait, macro hoặc module nằm trong cùng module, cùng crate, package khác, workspace khác hay dependency bên ngoài.

Tài liệu dùng cú pháp Rust hiện đại, ưu tiên **Rust Edition 2024**. Các ví dụ không phụ thuộc vào API không ổn định, trừ khi có ghi chú riêng. Phần mở rộng từ chương 39 trở đi trình bày cách tổ chức module cho **HTTP/REST API, MQTT, TCP, UDP, WebSocket, TLS, Tokio async, JSON, retry, timeout và graceful shutdown**. Các endpoint Internet trong ví dụ chỉ nhằm minh họa; bộ kiểm thử đi kèm có HTTP fixture và socket local để chạy mà không cần phụ thuộc dịch vụ bên ngoài.

---

## Workbook thực hành đi kèm

Để chuyển từ đọc lý thuyết sang thực hành, sử dụng workbook **[rust-practice-workbook.md](rust-practice-workbook.md)**. Workbook gồm 37 bài từ module/path/visibility, package/crate/workspace, feature, macro, build script, test và rustdoc đến Tokio, HTTP/REST, MQTT, TCP/UDP, WebSocket, TLS, retry, timeout, logging, bảo mật và project Device Gateway tích hợp. Sau đó học tiếp **[rust-production-workbook.md](rust-production-workbook.md)** với SQLx/SQLite, migration, transaction/outbox, Argon2id, JWT, authorization, property testing, benchmark, fault injection, observability, Docker, MQTT broker reliability, load test, threat model và capstone production. Bộ lời giải chạy được nằm trong thư mục `rust-practice-solutions/`; kiểm tra toàn bộ bằng `./scripts/verify.sh`.

---

## 1. Bức tranh tổng quát: package, crate, module và item

Rust có nhiều khái niệm gần nhau nhưng không đồng nghĩa. Phần lớn lỗi “không gọi được module”, “unresolved import” hoặc “private module” xuất hiện vì nhầm lẫn giữa các tầng này.

| Khái niệm | Ý nghĩa | Ví dụ hoặc nơi khai báo |
|---|---|---|
| **Package** | Đơn vị dự án do Cargo quản lý; có `Cargo.toml`, có thể chứa một hoặc nhiều crate | Thư mục có `Cargo.toml` |
| **Crate** | Đơn vị mã nguồn được compiler biên dịch tại một lần; có thể là binary crate hoặc library crate | `src/main.rs`, `src/lib.rs`, `src/bin/tool.rs` |
| **Crate root** | File bắt đầu của một crate, tạo ra module gốc của crate | `src/main.rs` hoặc `src/lib.rs` |
| **Module** | Không gian tên để nhóm mã và kiểm soát visibility | `mod user;`, `pub mod api { ... }` |
| **Item** | Thành phần được đặt trong module: function, struct, enum, trait, constant, static, type alias, module, macro… | `pub fn`, `struct`, `enum`, `trait` |
| **Path** | Chuỗi định danh dùng để chỉ tới một item trong cây module | `crate::domain::User` |
| **Workspace** | Nhóm nhiều package dùng chung một số cấu hình và lệnh Cargo | `[workspace]` trong `Cargo.toml` |

Theo tài liệu chính thức, crate là lượng mã nhỏ nhất mà compiler Rust xem xét trong một lần; crate có thể là binary hoặc library. Package là gói gồm một hoặc nhiều crate cùng một `Cargo.toml`, phải chứa ít nhất một crate và có nhiều nhất một library crate nhưng có thể có nhiều binary crate. [1]

Có thể hình dung mối quan hệ như sau:

```text
workspace
├── package-a
│   ├── library crate
│   │   └── module tree
│   └── binary crate(s)
└── package-b
    └── library crate
```

Còn trong một crate, cây module có dạng:

```text
crate
├── api
│   ├── request
│   └── response
├── domain
│   └── user
└── internal
```

**Tên thư mục không tự động tạo module.** Rust chỉ đưa file vào cây module khi bạn khai báo module bằng `mod`, hoặc khi Cargo nhận diện file đó là một crate root/target theo quy ước của Cargo.

---

## 2. Tạo package Rust bằng Cargo

Tạo một binary package:

```bash
cargo new hello_modules
cd hello_modules
cargo run
```

Cấu trúc mặc định:

```text
hello_modules/
├── Cargo.toml
└── src/
    └── main.rs
```

Tạo một library package:

```bash
cargo new --lib math_lib
```

Cấu trúc:

```text
math_lib/
├── Cargo.toml
└── src/
    └── lib.rs
```

`Cargo.toml` là manifest của package. Cargo dùng nó để biết tên package, phiên bản, edition, các target và dependency. Các mục thường gặp gồm `[package]`, `[lib]`, `[[bin]]`, `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[features]` và `[workspace]`. [2]

Một package có cả library và binary:

```text
my_app/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── main.rs
```

Trong trường hợp này có **hai crate**:

| File | Loại crate | Cách crate khác trong package gọi |
|---|---|---|
| `src/lib.rs` | Library crate | Dùng tên library crate, thường mặc định bằng tên package |
| `src/main.rs` | Binary crate | Đây là chương trình chạy được; gọi library qua tên crate |

Nếu package có các file sau:

```text
src/
├── main.rs
└── bin/
    ├── import.rs
    └── export.rs
```

thì package có một binary crate từ `main.rs` và hai binary crate bổ sung từ `src/bin/import.rs` và `src/bin/export.rs`. Chạy từng binary bằng:

```bash
cargo run --bin import
cargo run --bin export
```

---

## 3. Module root và cây module

Nội dung của `src/main.rs` hoặc `src/lib.rs` tạo thành module gốc của crate. Rust gọi module gốc này bằng từ khóa đặc biệt `crate`.

Ví dụ đơn giản trong `src/lib.rs`:

```rust
pub fn public_function() {
    println!("public function");
}

fn private_function() {
    println!("private function");
}

mod tools {
    pub fn format_name(name: &str) -> String {
        name.trim().to_uppercase()
    }
}
```

Cây module tương ứng:

```text
crate
├── public_function
├── private_function
└── tools
    └── format_name
```

`public_function` nằm ở `crate::public_function`; `tools::format_name` nằm ở `crate::tools::format_name`. Tuy nhiên, `tools` là private vì chỉ khai báo `mod tools`, nên code bên ngoài crate không thể truy cập nó, dù `format_name` bên trong có `pub`.

### 3.1. Module inline

Module có thể viết trực tiếp trong file:

```rust
mod calculator {
    pub fn add(left: i32, right: i32) -> i32 {
        left + right
    }

    pub fn subtract(left: i32, right: i32) -> i32 {
        left - right
    }
}

fn main() {
    let value = calculator::add(10, 5);
    println!("{value}");
}
```

Module inline phù hợp với nhóm mã nhỏ. Khi module lớn, nên tách sang file để dễ duy trì.

### 3.2. Module trong file riêng: kiểu hiện đại

Cấu trúc:

```text
src/
├── main.rs
└── calculator.rs
```

`src/main.rs`:

```rust
mod calculator;

fn main() {
    let value = calculator::add(10, 5);
    println!("{value}");
}
```

`src/calculator.rs`:

```rust
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

Dòng `mod calculator;` nói với compiler rằng module `calculator` được định nghĩa ở file `src/calculator.rs` hoặc `src/calculator/mod.rs`. Dòng này không phải là import theo nghĩa của `use`; nó là **khai báo và đưa module vào cây module**.

### 3.3. Module con trong thư mục

Cấu trúc hiện đại nên dùng:

```text
src/
├── main.rs
├── domain.rs
└── domain/
    ├── user.rs
    └── order.rs
```

`src/main.rs`:

```rust
mod domain;

fn main() {
    let user = domain::user::User::new("An");
    println!("{}", user.name());
}
```

`src/domain.rs`:

```rust
pub mod user;
pub mod order;
```

`src/domain/user.rs`:

```rust
pub struct User {
    name: String,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
```

`src/domain/order.rs`:

```rust
pub struct Order {
    pub id: u64,
}
```

Cây module là:

```text
crate
└── domain
    ├── user
    │   └── User
    └── order
        └── Order
```

Đường dẫn `crate::domain::user::User` chỉ hợp lệ nếu tất cả các module trên đường đi đều có khả năng truy cập. Vì vậy, `domain.rs` cần có `pub mod user;` nếu code bên ngoài `domain` phải nhìn thấy module `user`.

### 3.4. Kiểu thư mục cũ dùng `mod.rs`

Rust vẫn hỗ trợ:

```text
src/
├── main.rs
└── domain/
    ├── mod.rs
    └── user.rs
```

`src/domain/mod.rs`:

```rust
pub mod user;
```

`src/main.rs`:

```rust
mod domain;

fn main() {
    let user = domain::user::User::new("An");
    println!("{}", user.name());
}
```

Hai cách sau cùng biểu diễn cùng một module `domain`:

```text
src/domain.rs
```

hoặc:

```text
src/domain/mod.rs
```

Không nên đồng thời có cả `src/domain.rs` và `src/domain/mod.rs` cho cùng một module, vì compiler sẽ báo module được định nghĩa nhiều lần hoặc không biết chọn file nào.

### 3.5. Dùng `#[path = "..."]`

Có thể chỉ định file bất kỳ làm nguồn của module:

```rust
#[path = "generated/calculator.rs"]
mod calculator;

fn main() {
    println!("{}", calculator::add(2, 3));
}
```

Ví dụ này yêu cầu file:

```text
src/generated/calculator.rs
```

`#[path]` hữu ích trong một số cấu trúc đặc biệt hoặc khi tương thích với mã sinh tự động, nhưng không nên lạm dụng trong thiết kế thông thường vì làm module tree khó đoán hơn.

### 3.6. Khác biệt giữa `mod` và `use`

Đây là phân biệt nền tảng:

| Cú pháp | Vai trò |
|---|---|
| `mod payments;` | Khai báo module và yêu cầu compiler biên dịch file/module đó |
| `use crate::payments::Card;` | Tạo tên tắt cho item đã tồn tại trong cây module |
| `pub mod payments;` | Khai báo module và cho phép module cha/bên ngoài truy cập theo quy tắc visibility |
| `pub use crate::payments::Card;` | Đưa item vào scope hiện tại đồng thời tái xuất nó cho code bên ngoài |

Ví dụ:

```rust
mod payments;

use payments::Card;

fn main() {
    let _card = Card::new();
}
```

Nếu xóa `mod payments;`, dòng `use payments::Card;` không thể tự tìm `payments.rs`. `use` không thay thế cho `mod`.

---

## 4. Path: mọi cách chỉ tới một item

Một path gồm các định danh nối bằng `::`, tương tự đường dẫn thư mục. Rust có path tuyệt đối và path tương đối. [3]

### 4.1. Path tuyệt đối bắt đầu bằng `crate`

Trong cùng một crate, path tuyệt đối bắt đầu bằng `crate`:

```rust
mod domain {
    pub mod user {
        pub fn create() {
            println!("create user");
        }
    }
}

fn run() {
    crate::domain::user::create();
}

fn main() {
    run();
}
```

`crate::domain::user::create()` luôn bắt đầu từ module root của crate hiện tại. Đây thường là lựa chọn rõ ràng khi module gọi và module được gọi có thể được di chuyển độc lập.

### 4.2. Path tương đối bắt đầu bằng tên module hiện tại

```rust
mod domain {
    pub mod user {
        pub fn create() {
            println!("create user");
        }
    }

    pub fn create_default_user() {
        user::create();
    }
}
```

Trong `create_default_user`, `user::create()` là path tương đối: Rust tìm module `user` từ module hiện tại là `domain`.

### 4.3. `self::` — module hiện tại

```rust
mod app {
    pub fn start() {
        self::initialize();
    }

    fn initialize() {
        println!("initialized");
    }
}
```

`self::initialize()` chỉ tới item trong module hiện tại. Trong nhiều trường hợp có thể bỏ `self::` và viết trực tiếp `initialize()`, nhưng `self::` hữu ích khi cần làm rõ điểm bắt đầu của path hoặc khi dùng trong nested `use`.

```rust
use std::io::{self, Read};
```

Ở đây `self` đưa `std::io` vào scope và `Read` đưa trait `std::io::Read` vào scope.

### 4.4. `super::` — module cha

```rust
fn deliver_order() {
    println!("delivered");
}

mod kitchen {
    pub fn fix_order() {
        super::deliver_order();
    }
}

fn main() {
    kitchen::fix_order();
}
```

`super::deliver_order()` đi lên module cha của `kitchen`, tức module root. Nó phù hợp khi module con có quan hệ chặt với module cha và nên di chuyển cùng nhau.

Có thể đi lên nhiều cấp:

```rust
super::super::shared::log();
```

Tuy nhiên, path đi lên quá nhiều cấp thường cho thấy module tree đang bị liên kết chặt; nên cân nhắc một API hoặc re-export ổn định hơn.

### 4.5. Path tương đối bằng tên trực tiếp

```rust
mod a {
    pub mod b {
        pub fn hello() {}
    }
}

fn main() {
    a::b::hello();
}
```

Tại `main`, `a::b::hello()` là path tương đối đối với module root. Trong code bên trong module `c`, một path không bắt đầu bằng `crate`, `self`, `super` hoặc tên crate ngoài sẽ được phân giải theo scope và module hiện tại.

### 4.6. Gọi item qua `crate`

```rust
mod service {
    pub fn run() {
        crate::repository::load();
    }
}

mod repository {
    pub fn load() {
        println!("loaded");
    }
}
```

Đây là cách phổ biến để module trong cùng library crate gọi nhau:

```rust
crate::service::run();
crate::repository::load();
```

### 4.7. Gọi từ binary sang library trong cùng package

Giả sử `Cargo.toml` có:

```toml
[package]
name = "shop_app"
version = "0.1.0"
edition = "2024"
```

`src/lib.rs`:

```rust
pub mod shop {
    pub fn open() {
        println!("shop is open");
    }
}
```

`src/main.rs`:

```rust
use shop_app::shop;

fn main() {
    shop::open();
}
```

Điểm quan trọng là trong `src/main.rs`, `crate::shop` chỉ trỏ tới module root của **binary crate**, không trỏ tới `src/lib.rs`. Binary phải gọi library crate qua tên library crate, ở đây là `shop_app`.

Nếu package name có dấu gạch ngang:

```toml
[package]
name = "my-shop-app"
version = "0.1.0"
edition = "2024"
```

thì tên được dùng trong path Rust thường chuyển dấu `-` thành `_`:

```rust
use my_shop_app::shop;
```

Khi cần chắc chắn tên library target, có thể khai báo:

```toml
[lib]
name = "shop_core"
path = "src/lib.rs"
```

Khi đó binary gọi:

```rust
use shop_core::shop;
```

### 4.8. Gọi dependency bên ngoài

Sau khi thêm dependency:

```toml
[dependencies]
serde = "1"
```

code có thể dùng path bắt đầu bằng tên crate:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
}
```

Hoặc gọi trực tiếp:

```rust
let json = serde_json::to_string(&user)?;
```

`std` cũng là một crate bên ngoài package nhưng được phân phối cùng toolchain Rust, nên không cần thêm vào `[dependencies]`:

```rust
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
```

Cargo hỗ trợ dependency từ registry, Git repository, local path, registry khác, dependency theo nền tảng, dependency dành cho test và dependency dành cho build script. [4]

### 4.9. Associated function, method, associated constant và trait item

Không phải mọi cách gọi dùng `::` đều là module path. `::` còn dùng để gọi associated item:

```rust
struct User {
    id: u64,
}

impl User {
    const DEFAULT_ID: u64 = 0;

    fn new(id: u64) -> Self {
        Self { id }
    }

    fn id(&self) -> u64 {
        self.id
    }
}

fn main() {
    let user = User::new(User::DEFAULT_ID);
    println!("{}", user.id());
}
```

Trong ví dụ này:

```text
User::new(...)       // associated function
User::DEFAULT_ID    // associated constant
user.id()            // method
```

Với trait có associated function hoặc method:

```rust
trait Describe {
    fn describe(&self) -> String;

    fn type_name() -> &'static str
    where
        Self: Sized;
}

struct User;

impl Describe for User {
    fn describe(&self) -> String {
        "user".to_owned()
    }

    fn type_name() -> &'static str {
        "User"
    }
}

fn main() {
    let user = User;
    println!("{}", user.describe());
    println!("{}", User::type_name());
}
```

Khi nhiều trait có item trùng tên, dùng fully qualified syntax:

```rust
trait A {
    fn name() -> &'static str;
}

trait B {
    fn name() -> &'static str;
}

struct Item;

impl A for Item {
    fn name() -> &'static str {
        "A"
    }
}

impl B for Item {
    fn name() -> &'static str {
        "B"
    }
}

fn main() {
    println!("{}", <Item as A>::name());
    println!("{}", <Item as B>::name());
}
```

### 4.10. Gọi enum variant

```rust
enum Status {
    Ready,
    Failed(String),
}

fn main() {
    let status = Status::Ready;
    let error = Status::Failed("network error".to_owned());

    match error {
        Status::Ready => println!("ready"),
        Status::Failed(message) => println!("{message}"),
    }
}
```

Có thể import variant riêng:

```rust
use Status::{Failed, Ready};
```

Hoặc import tất cả variant:

```rust
use Status::*;
```

Glob cho enum nên dùng có kiểm soát để tránh làm scope khó đọc.

---

## 5. Visibility và privacy: `pub` không có nghĩa là “mọi nơi đều gọi được”

Mặc định item của Rust là private. Hai ngoại lệ quan trọng là associated item trong `pub trait` và variant của `pub enum` được public theo quy tắc tương ứng. [5]

### 5.1. Module public và item public là hai việc khác nhau

Ví dụ sai:

```rust
pub mod api {
    fn run() {}
}

fn main() {
    api::run(); // lỗi: run private
}
```

Module `api` public nhưng `run` vẫn private. Cách đúng:

```rust
pub mod api {
    pub fn run() {}
}

fn main() {
    api::run();
}
```

Nếu module cha private, item con public cũng chưa đủ để code bên ngoài crate gọi:

```rust
mod internal {
    pub fn visible_only_inside_crate_boundary() {}
}
```

Code bên ngoài không thể gọi `internal::visible_only_inside_crate_boundary()` vì `internal` không public. Trong cùng crate, các module khác vẫn có thể truy cập item public nằm trong module private nếu quy tắc privacy cho phép.

### 5.2. Chuỗi visibility phải thông suốt

Muốn external crate gọi:

```rust
crate::a::b::run()
```

thì thường cần:

```rust
pub mod a {
    pub mod b {
        pub fn run() {}
    }
}
```

Nếu bất kỳ mắt xích cần thiết nào private, path public đó không thể được dùng từ vị trí bên ngoài mắt xích đó. Rust Reference mô tả rằng item public chỉ có thể được truy cập khi các module tổ tiên trên đường đi cũng truy cập được; item private được module hiện tại và các module con của nó sử dụng. [5]

### 5.3. `pub(crate)`

`pub(crate)` cho phép dùng trong toàn bộ crate hiện tại nhưng không công khai cho crate phụ thuộc:

```rust
pub(crate) mod internal_api {
    pub(crate) fn reset_cache() {}
}

fn run() {
    crate::internal_api::reset_cache();
}
```

Đây là lựa chọn tốt khi cần chia sẻ giữa nhiều module nội bộ nhưng không muốn biến thành public API của library.

### 5.4. `pub(super)`

`pub(super)` chỉ cho module cha truy cập:

```rust
mod parent {
    mod child {
        pub(super) fn only_parent_can_call() {}
    }

    pub fn run() {
        child::only_parent_can_call();
    }
}
```

Module khác ngang cấp với `parent` không được gọi hàm này.

### 5.5. `pub(self)`

`pub(self)` chỉ công khai trong module hiện tại, tương đương về thực tế với không ghi `pub` trong nhiều trường hợp:

```rust
mod parser {
    pub(self) fn parse_token() {}
}
```

Nó chủ yếu có giá trị biểu đạt hoặc dùng khi viết macro/generator cần visibility tường minh.

### 5.6. `pub(in path)`

Giới hạn visibility tới một module tổ tiên cụ thể:

```rust
mod application {
    pub mod service {
        pub(in crate::application) fn helper() {}

        pub fn run() {
            helper();
        }
    }

    pub fn call_service_helper() {
        service::helper();
    }
}
```

Trong Edition 2018 trở lên, path của `pub(in path)` phải bắt đầu bằng `crate`, `self` hoặc `super`. `pub(in crate::application)` làm item nhìn thấy trong phạm vi module `crate::application`. [5]

Các dạng cần nhớ:

| Cú pháp | Phạm vi nhìn thấy |
|---|---|
| Không có `pub` | Module hiện tại và các module con theo quy tắc private |
| `pub` | Nơi bên ngoài có thể truy cập nếu toàn bộ ancestor path cũng truy cập được |
| `pub(crate)` | Toàn bộ crate hiện tại |
| `pub(super)` | Module cha trực tiếp |
| `pub(self)` | Module hiện tại |
| `pub(in crate::x)` | Module `crate::x` và các module con trong đó |

### 5.7. Struct public nhưng field private

```rust
pub struct Account {
    pub id: u64,
    balance: i64,
}

impl Account {
    pub fn new(id: u64, balance: i64) -> Self {
        Self { id, balance }
    }

    pub fn balance(&self) -> i64 {
        self.balance
    }
}
```

Code bên ngoài có thể đọc/ghi `account.id`, nhưng không thể đọc/ghi trực tiếp `account.balance`. Vì field private, API nên cung cấp constructor và method public.

```rust
let account = Account::new(1, 500);
println!("{}", account.id);
println!("{}", account.balance());
// account.balance = 1000; // lỗi: field private
```

Nếu struct có field private, code bên ngoài cũng không thể tự tạo literal đầy đủ nếu không có constructor public phù hợp.

### 5.8. Public enum và variant

```rust
pub enum Color {
    Red,
    Green,
    Blue,
}
```

Các variant `Color::Red`, `Color::Green`, `Color::Blue` có thể dùng từ code bên ngoài vì enum public và variant enum public theo quy tắc của Rust.

### 5.9. Trait phải được đưa vào scope để gọi method

```rust
use std::fmt::Write as _;

fn main() {
    let mut output = String::new();
    write!(&mut output, "value = {}", 42).unwrap();
    println!("{output}");
}
```

Một trait thường cần được import để method extension của nó xuất hiện trong method resolution:

```rust
use std::io::Read;

fn read_all(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}
```

Nếu bỏ `use std::io::Read;`, compiler có thể không tìm thấy method `read_to_end` trên kiểu đang dùng, dù kiểu đó triển khai trait `Read`.

---

## 6. `use`: tạo tên tắt trong scope

`use` tạo shortcut cho một path trong **scope nơi câu lệnh `use` xuất hiện**. Nó không sao chép item, không tạo module mới và không làm item trở nên public. [6]

### 6.1. Import module

```rust
mod frontend {
    pub mod http {
        pub fn get() {}
    }
}

use crate::frontend::http;

fn main() {
    http::get();
}
```

### 6.2. Import type, enum, trait và constant

```rust
use std::collections::HashMap;
use std::fmt::Display;
use std::time::Duration;

fn print_value<T: Display>(value: T) {
    println!("{value}");
}

fn main() {
    let mut map = HashMap::new();
    map.insert("language", "Rust");
    print_value(Duration::from_secs(1).as_secs());
}
```

Với struct, enum và type, kiểu viết đầy đủ thường được xem là rõ ràng:

```rust
use std::collections::HashMap;
```

### 6.3. Import function: module hay function?

Cả hai đều hợp lệ:

```rust
mod restaurant {
    pub mod kitchen {
        pub fn cook() {}
    }
}

use crate::restaurant::kitchen;

fn main() {
    kitchen::cook();
}
```

Hoặc:

```rust
use crate::restaurant::kitchen::cook;

fn main() {
    cook();
}
```

Cách đầu thường dễ đọc hơn vì cho thấy `cook` thuộc module `kitchen`, tránh nhầm với function local. Với type/struct/enum, import trực tiếp type thường là quy ước phổ biến:

```rust
use crate::restaurant::kitchen::Chef;
```

### 6.4. `as`: đổi tên import

```rust
use std::fmt::Result;
use std::io::Result as IoResult;

fn format_value() -> Result {
    Ok(())
}

fn read_value() -> IoResult<()> {
    Ok(())
}
```

Có thể alias dependency:

```rust
use serde_json as json;

fn encode(value: &str) -> String {
    json::to_string(value).unwrap()
}
```

### 6.5. `use` trong scope cục bộ

```rust
fn run() {
    {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert("Rust");
        println!("{set:?}");
    }

    // HashSet không còn trong scope ở đây.
}
```

Một `use` ở module cha không tự động xuất hiện trong module con:

```rust
use std::collections::HashMap;

mod child {
    pub fn run() {
        // HashMap không tự động có mặt ở đây.
        let _ = std::collections::HashMap::<String, i32>::new();
    }
}
```

Muốn dùng tên ngắn trong child module, import lại:

```rust
mod child {
    use std::collections::HashMap;

    pub fn run() {
        let _ = HashMap::<String, i32>::new();
    }
}
```

Hoặc dùng shortcut từ module cha bằng `super::` nếu shortcut đó là tên có thể truy cập:

```rust
use std::collections::HashMap;

mod child {
    pub fn run() {
        let _ = super::HashMap::<String, i32>::new();
    }
}
```

### 6.6. Nested paths

```rust
use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{self, Read},
};
```

Tương đương:

```rust
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::io::Read;
```

Có thể dùng `self` để import cả module và item con:

```rust
use std::io::{self, Read, Write};
```

### 6.7. Glob import `*`

```rust
use std::collections::*;
```

Lệnh này đưa các public item của `std::collections` vào scope. Glob tiện trong một số test hoặc module tiền xử lý, nhưng trong code production lớn thường không nên lạm dụng vì khó biết một tên đến từ đâu và dễ xung đột khi dependency thêm item mới. [6]

### 6.8. Import macro

Macro khai báo bằng `macro_rules!` có thể được import theo quy tắc hiện đại:

```rust
mod macros {
    #[macro_export]
    macro_rules! announce {
        ($message:expr) => {
            println!("ANNOUNCE: {}", $message);
        };
    }
}

fn main() {
    announce!("hello");
}
```

`#[macro_export]` đặt macro trong root namespace của crate để crate khác có thể gọi bằng tên crate. Với macro từ dependency, cách gọi thường là:

```rust
use tracing::info;

fn main() {
    info!("started");
}
```

Một số macro cũ hoặc macro procedural có thể có quy tắc import riêng; luôn xem tài liệu crate và thông báo compiler.

### 6.9. `pub use`: tái xuất API

`use` chỉ tạo tên trong scope hiện tại. `pub use` vừa import vừa làm tên đó trở thành một phần API của module hiện tại:

```rust
mod implementation {
    pub mod user {
        pub struct User {
            pub id: u64,
        }
    }
}

pub use implementation::user::User;
```

Code bên ngoài có thể viết:

```rust
use my_library::User;
```

thay vì:

```rust
use my_library::implementation::user::User;
```

Đây là kỹ thuật **facade module** hoặc **public API re-export**. Nó cho phép module nội bộ được tổ chức theo cách thuận tiện cho tác giả, trong khi người dùng thấy API ngắn và ổn định:

```rust
// src/lib.rs
mod domain;
mod infrastructure;

pub use domain::{Order, User};
pub use infrastructure::Client;
```

Nên re-export những item chủ chốt mà người dùng thường xuyên cần. Không nên re-export toàn bộ cấu trúc nội bộ một cách tùy tiện vì sẽ làm public API khó kiểm soát.

### 6.10. Re-export với alias

```rust
mod internal {
    pub struct VeryLongInternalType;
}

pub use internal::VeryLongInternalType as PublicType;
```

Bên ngoài dùng:

```rust
use my_library::PublicType;
```

### 6.11. Re-export dependency

Một library có thể expose một kiểu từ dependency:

```rust
// Cargo.toml
[dependencies]
http = "1"

// src/lib.rs
pub use http::StatusCode;
```

Người dùng library có thể dùng `my_library::StatusCode` thay vì tự thêm dependency `http` trong một số thiết kế API. Tuy nhiên, re-export dependency làm dependency trở thành một phần của public API; cần cân nhắc tương thích phiên bản.

---

## 7. Package và dependency: cách gọi package khác

### 7.1. Dependency từ crates.io

```toml
[package]
name = "weather_app"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Trong Rust:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Weather {
    city: String,
    temperature: f32,
}

fn main() {
    let weather = Weather {
        city: "Hanoi".to_owned(),
        temperature: 30.5,
    };

    let json = serde_json::to_string(&weather).unwrap();
    println!("{json}");
}
```

Tên dependency ở `Cargo.toml` thường trùng tên crate import trong Rust, nhưng tên package và tên library target có thể được cấu hình khác nhau.

### 7.2. Dependency local bằng `path`

Cấu trúc:

```text
workspace-root/
├── Cargo.toml
├── app/
│   ├── Cargo.toml
│   └── src/main.rs
└── shared/
    ├── Cargo.toml
    └── src/lib.rs
```

`app/Cargo.toml`:

```toml
[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
shared = { path = "../shared" }
```

`shared/src/lib.rs`:

```rust
pub fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}
```

`app/src/main.rs`:

```rust
use shared::greeting;

fn main() {
    println!("{}", greeting("Rust"));
}
```

`path` phải trỏ tới đúng thư mục có `Cargo.toml` của dependency; Cargo không tự đi xuyên cây thư mục local để tìm package như với một số Git repository. [4]

### 7.3. Dependency Git

```toml
[dependencies]
my_library = { git = "https://github.com/example/my_library.git" }
```

Khóa rõ branch, tag hoặc revision:

```toml
[dependencies]
my_library = {
    git = "https://github.com/example/my_library.git",
    tag = "v1.2.0",
}
```

Hoặc:

```toml
[dependencies]
my_library = {
    git = "https://github.com/example/my_library.git",
    rev = "0123456789abcdef0123456789abcdef01234567",
}
```

### 7.4. Đổi tên dependency bằng `package`

Giả sử package thật có tên `common-types`, nhưng muốn gọi nó trong code là `types`:

```toml
[dependencies]
types = { package = "common-types", version = "1" }
```

Rust:

```rust
use types::UserId;
```

Nếu có hai dependency cùng package name nhưng đến từ các nguồn khác nhau, alias giúp tách tên:

```toml
[dependencies]
old_types = { package = "common-types", version = "1" }
git_types = {
    package = "common-types",
    git = "https://github.com/example/common-types.git",
}
```

Rust:

```rust
use old_types::UserId as OldUserId;
use git_types::UserId as GitUserId;
```

Cargo dùng khóa `package` để chỉ package thực sự được chọn, còn khóa bên trái là tên dependency dùng trong manifest và thường là tên crate dùng trong source. [4]

### 7.5. Dependency chỉ dùng khi phát triển

```toml
[dev-dependencies]
pretty_assertions = "1"
```

Dependency này dành cho test, example và benchmark; không trở thành dependency runtime được truyền tiếp cho package khác. [4]

### 7.6. Dependency cho build script

```toml
[build-dependencies]
cc = "1"
```

`build.rs` được biên dịch riêng, nên dependency trong `[build-dependencies]` chỉ có mặt cho build script. Nếu source package cũng cần crate đó, phải khai báo thêm trong `[dependencies]`. [4]

### 7.7. Dependency optional và feature

```toml
[dependencies]
serde = { version = "1", optional = true, features = ["derive"] }

[features]
default = []
serde-support = ["dep:serde"]
```

Trong code:

```rust
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};
```

Conditional module:

```rust
#[cfg(feature = "serde-support")]
pub mod serialization;
```

Chạy với feature:

```bash
cargo check --features serde-support
cargo check --no-default-features
```

### 7.8. Dependency theo nền tảng

```toml
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

Code cũng có thể phân nhánh:

```rust
#[cfg(unix)]
fn platform_name() -> &'static str {
    "unix"
}

#[cfg(windows)]
fn platform_name() -> &'static str {
    "windows"
}
```

---

## 8. `extern crate`: khi nào còn dùng?

Trong Rust Edition 2018 trở lên, dependency trong `Cargo.toml` thường tự có trong extern prelude, nên không cần:

```rust
extern crate serde;
```

Chỉ cần:

```rust
use serde::Serialize;
```

`extern crate` chủ yếu gặp trong mã Edition 2015, một số tình huống tương thích cũ hoặc khi cần đưa macro/extern crate vào scope theo cách đặc biệt. Code mới nên ưu tiên `Cargo.toml` + path crate + `use`.

---

## 9. Library crate và binary crate trong cùng package

Đây là mẫu tổ chức khuyến nghị cho ứng dụng có logic muốn tái sử dụng hoặc test dễ dàng.

`src/lib.rs`:

```rust
pub mod domain {
    pub struct User {
        name: String,
    }

    impl User {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }
}

pub fn greeting(user: &domain::User) -> String {
    format!("Hello, {}", user.name())
}
```

`src/main.rs`:

```rust
use my_app::{domain::User, greeting};

fn main() {
    let user = User::new("An");
    println!("{}", greeting(&user));
}
```

Trong `main.rs`, không viết `use crate::domain::User` để lấy module từ `lib.rs`. `crate` tại đó là binary crate. Hãy dùng tên library crate:

```rust
use my_app::domain::User;
```

Nếu cần đổi tên library target:

```toml
[lib]
name = "core_logic"
path = "src/lib.rs"
```

thì dùng:

```rust
use core_logic::domain::User;
```

Mẫu này giúp binary chỉ làm nhiệm vụ khởi động, phân tích argument hoặc wiring; logic chính ở library có thể được integration test và tái sử dụng. Rust Book cũng mô tả đây là cách tổ chức phổ biến cho package chứa cả binary và library. [3]

---

## 10. Workspace: nhiều package gọi nhau

Workspace là tập hợp các package được Cargo quản lý cùng nhau. Các package chia sẻ `Cargo.lock`, thư mục output mặc định và có thể chạy lệnh chung như `cargo check --workspace`. [7]

Root `Cargo.toml` dạng virtual workspace:

```toml
[workspace]
members = ["apps/cli", "crates/core", "crates/utils"]
resolver = "3"
```

Cấu trúc:

```text
project/
├── Cargo.toml
├── apps/
│   └── cli/
│       ├── Cargo.toml
│       └── src/main.rs
└── crates/
    ├── core/
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── utils/
        ├── Cargo.toml
        └── src/lib.rs
```

`apps/cli/Cargo.toml`:

```toml
[package]
name = "cli"
version = "0.1.0"
edition = "2024"

[dependencies]
core = { path = "../../crates/core" }
utils = { path = "../../crates/utils" }
```

`apps/cli/src/main.rs`:

```rust
use core::run;
use utils::format_title;

fn main() {
    println!("{}", format_title("CLI"));
    run();
}
```

Chạy:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p cli
cargo check -p core
```

### 10.1. Chia sẻ dependency trong workspace

Root `Cargo.toml`:

```toml
[workspace]
members = ["apps/cli", "crates/core"]
resolver = "3"

[workspace.dependencies]
thiserror = "2"
serde = { version = "1", features = ["derive"] }
```

Member `crates/core/Cargo.toml`:

```toml
[package]
name = "core"
version = "0.1.0"
edition = "2024"

[dependencies]
thiserror.workspace = true
serde.workspace = true
```

Member có thể bổ sung feature mang tính additive:

```toml
[dependencies]
serde = { workspace = true, features = ["alloc"] }
```

### 10.2. Đặc điểm quan trọng của workspace

Workspace không tự tạo quan hệ phụ thuộc giữa các package. Hai package cùng là member không có nghĩa package này tự động gọi được package kia; vẫn phải khai báo `[dependencies]` bằng `path`, version hoặc nguồn phù hợp.

Có thể chạy tất cả member:

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
```

---

## 11. Mẫu module tree thực tế cho library

Một cấu trúc dễ mở rộng:

```text
src/
├── lib.rs
├── error.rs
├── config.rs
├── domain.rs
├── domain/
│   ├── user.rs
│   └── order.rs
├── application.rs
├── application/
│   ├── user_service.rs
│   └── order_service.rs
├── infrastructure.rs
└── infrastructure/
    ├── database.rs
    └── http.rs
```

`src/lib.rs`:

```rust
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;

pub use application::{OrderService, UserService};
pub use config::Config;
pub use domain::{Order, User};
pub use error::AppError;
```

`src/domain.rs`:

```rust
mod order;
mod user;

pub use order::Order;
pub use user::User;
```

`src/application.rs`:

```rust
mod order_service;
mod user_service;

pub use order_service::OrderService;
pub use user_service::UserService;
```

Mô hình này giữ module triển khai chi tiết ở private path, nhưng public API của library nằm ở root:

```rust
use my_library::{Config, User, UserService};
```

Thay vì buộc người dùng biết toàn bộ cấu trúc:

```rust
use my_library::application::user_service::UserService;
use my_library::domain::user::User;
```

### 11.1. `pub mod` hay `mod` + `pub use`?

Dùng `pub mod` khi module đó thực sự là một namespace người dùng cần truy cập:

```rust
pub mod prelude;
pub mod error;
```

Dùng `mod` và `pub use` khi muốn giấu cấu trúc nội bộ:

```rust
mod domain;
pub use domain::User;
```

Trong library lớn, `mod` + `pub use` thường tạo public API ổn định hơn. Bạn có thể đổi file `domain/user.rs` thành `domain/customer.rs` mà không buộc người dùng đổi `use my_library::User`.

### 11.2. Prelude module

Một số library cung cấp module `prelude`:

```rust
pub mod prelude {
    pub use crate::{Config, User, UserService};
}
```

Người dùng viết:

```rust
use my_library::prelude::*;
```

Prelude nên chỉ chứa các trait/type được dùng rất thường xuyên. Không nên đưa mọi public item vào prelude vì sẽ gây xung đột tên và làm API kém minh bạch.

---

## 12. Test trong module và cách gọi item private

### 12.1. Unit test nằm trong cùng file/module

```rust
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

fn internal_value() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn can_test_private_function() {
        assert_eq!(internal_value(), 42);
    }
}
```

`super::*` import các item từ module cha. Unit test là module con nên có thể truy cập private item của module cha theo quy tắc privacy.

### 12.2. Integration test chỉ dùng public API

Cấu trúc:

```text
tests/
└── public_api.rs
```

`tests/public_api.rs`:

```rust
use my_library::User;

#[test]
fn creates_user_through_public_api() {
    let user = User::new("An");
    assert_eq!(user.name(), "An");
}
```

Integration test được biên dịch như crate bên ngoài, vì vậy không thể gọi private module hoặc private function. Đây là cách tốt để kiểm tra public API thật sự.

### 12.3. Example cũng là crate riêng

Cấu trúc:

```text
examples/
└── basic.rs
```

`examples/basic.rs`:

```rust
use my_library::User;

fn main() {
    let user = User::new("An");
    println!("{}", user.name());
}
```

Chạy:

```bash
cargo run --example basic
```

Vì example dùng library như một crate bên ngoài, example chỉ gọi được public API.

---

## 13. `include!`, `include_str!` và `include_bytes!` không phải module thông thường

`include!` chèn nội dung một file Rust vào vị trí macro được gọi:

```rust
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/routes.rs"));
```

Nó khác `mod generated;` ở chỗ không tạo module namespace riêng theo cách thông thường; item được chèn vào scope hiện tại. Vì vậy, cần cẩn thận về tên trùng, span lỗi và visibility.

Dùng `include_str!` cho file text:

```rust
const VERSION: &str = include_str!("../VERSION.txt");
```

Dùng `include_bytes!` cho dữ liệu nhị phân:

```rust
static ICON: &[u8] = include_bytes!("../assets/icon.bin");
```

Nếu mục tiêu là tổ chức source code, ưu tiên `mod`. Nếu mục tiêu là nhúng dữ liệu hoặc source sinh tự động, cân nhắc `include!`/`include_str!`/`include_bytes!`.

---

## 14. Các kiểu gọi thường gặp theo loại item

| Item | Khai báo ví dụ | Cách gọi |
|---|---|---|
| Function module | `pub fn run()` | `module::run()` |
| Struct | `pub struct User` | `User { ... }`, `User::new()` |
| Field public | `pub id: u64` | `user.id` |
| Method | `fn name(&self)` | `user.name()` |
| Associated function | `fn new(...) -> Self` | `User::new(...)` |
| Associated constant | `const MAX: usize` | `User::MAX` |
| Enum variant | `enum State { Ready }` | `State::Ready` |
| Trait method | `trait Log { fn log(...) }` | `value.log()` sau khi trait trong scope |
| Trait associated item | `trait Factory { fn create() }` | `<Type as Factory>::create()` |
| Constant module | `pub const VERSION: &str` | `module::VERSION` |
| Type alias | `pub type UserId = u64` | `module::UserId` |
| Macro | `macro_rules! log` | `log!(...)` |
| Dependency crate | `[dependencies] regex = "..."` | `regex::Regex::new(...)` |
| Library cùng package | `src/lib.rs` | `package_name::item` từ binary |
| Re-export | `pub use internal::User` | `crate::User` hoặc `library::User` |

Ví dụ tổng hợp:

```rust
pub mod api {
    pub const VERSION: &str = "1.0";

    pub struct User {
        pub id: u64,
    }

    impl User {
        pub const DEFAULT_ID: u64 = 0;

        pub fn new(id: u64) -> Self {
            Self { id }
        }

        pub fn id(&self) -> u64 {
            self.id
        }
    }
}

pub trait Identifiable {
    fn id(&self) -> u64;
}

impl Identifiable for api::User {
    fn id(&self) -> u64 {
        self.id()
    }
}

fn main() {
    let user = api::User::new(api::User::DEFAULT_ID);
    println!("{}", api::VERSION);
    println!("{}", user.id());
    println!("{}", <api::User as Identifiable>::id(&user));
}
```

---

## 15. Ví dụ hoàn chỉnh có thể chạy: library, module, re-export và binary

Tạo project:

```bash
cargo new rust_calling_demo
cd rust_calling_demo
```

Cấu trúc cuối:

```text
rust_calling_demo/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs
    ├── domain.rs
    ├── domain/
    │   ├── user.rs
    │   └── role.rs
    └── internal.rs
```

`src/lib.rs`:

```rust
mod domain;
mod internal;

pub use domain::{Role, User};

pub fn greet(user: &User) -> String {
    internal::audit("greet");
    format!("Hello, {} ({:?})", user.name(), user.role())
}
```

`src/domain.rs`:

```rust
mod role;
mod user;

pub use role::Role;
pub use user::User;
```

`src/domain/role.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Member,
}
```

`src/domain/user.rs`:

```rust
use super::Role;

#[derive(Debug)]
pub struct User {
    name: String,
    role: Role,
}

impl User {
    pub fn new(name: impl Into<String>, role: Role) -> Self {
        Self {
            name: name.into(),
            role,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }
}
```

`src/internal.rs`:

```rust
pub(crate) fn audit(event: &str) {
    println!("audit: {event}");
}
```

`src/main.rs`:

```rust
use rust_calling_demo::{Role, User, greet};

fn main() {
    let user = User::new("An", Role::Member);
    println!("{}", greet(&user));
}
```

Các điểm cần quan sát:

1. `domain` và `internal` là private đối với crate bên ngoài.
2. `User` và `Role` được đưa ra public API bằng `pub use` ở `lib.rs`.
3. `User` gọi `Role` từ module cha bằng `use super::Role;`.
4. `internal::audit` là `pub(crate)`, nên `lib.rs` dùng được nhưng binary bên ngoài library không gọi trực tiếp được.
5. `main.rs` dùng `rust_calling_demo::...` vì nó gọi library crate cùng package.
6. Người dùng library chỉ cần biết `rust_calling_demo::User`, không cần biết `domain::user::User`.

Chạy:

```bash
cargo check
cargo run
cargo test
```

---

## 16. Ví dụ hoàn chỉnh có thể chạy: workspace nhiều package

Tạo thư mục:

```bash
mkdir rust_workspace_demo
cd rust_workspace_demo
cargo new crates/math-core --lib
cargo new apps/calculator
```

Root `Cargo.toml`:

```toml
[workspace]
members = ["crates/math-core", "apps/calculator"]
resolver = "3"
```

`crates/math-core/Cargo.toml`:

```toml
[package]
name = "math-core"
version = "0.1.0"
edition = "2024"
```

`crates/math-core/src/lib.rs`:

```rust
pub mod operations {
    pub fn add(left: i64, right: i64) -> i64 {
        left + right
    }

    pub fn multiply(left: i64, right: i64) -> i64 {
        left * right
    }
}

pub use operations::{add, multiply};
```

`apps/calculator/Cargo.toml`:

```toml
[package]
name = "calculator"
version = "0.1.0"
edition = "2024"

[dependencies]
math-core = { path = "../../crates/math-core" }
```

`apps/calculator/src/main.rs`:

```rust
use math_core::{add, multiply};

fn main() {
    let sum = add(2, 3);
    let product = multiply(4, 5);
    println!("sum = {sum}, product = {product}");
}
```

Chạy từ root:

```bash
cargo run -p calculator
cargo test --workspace
```

Ở đây có hai package, mỗi package có một library hoặc binary crate. `calculator` không gọi package `math-core` bằng `crate::`; nó gọi library crate bằng tên dependency `math_core` theo quy tắc dấu gạch ngang thành dấu gạch dưới trong Rust path.

---

## 17. Lỗi thường gặp và cách sửa

### 17.1. `use of unresolved module or unlinked crate`

Sai:

```rust
use utils::format_name;
```

nhưng chưa có:

```toml
[dependencies]
utils = { path = "../utils" }
```

Cách sửa là thêm dependency đúng vào `Cargo.toml`, kiểm tra tên package/crate, rồi chạy:

```bash
cargo check
```

Nếu là module nội bộ, có thể bạn quên `mod utils;` trong crate root hoặc file không đúng vị trí.

### 17.2. `unresolved import crate::x`

Nguyên nhân thường gặp:

```rust
use crate::user::User;
```

nhưng không có `mod user;` trong root, hoặc `user` thực tế nằm trong `crate::domain::user`.

Kiểm tra cây module:

```rust
mod domain;

use crate::domain::user::User;
```

và trong `domain.rs`:

```rust
pub mod user;
```

### 17.3. `module is private` hoặc `function is private`

Sai:

```rust
mod api {
    pub fn run() {}
}

// từ external crate:
api::run();
```

Sửa:

```rust
pub mod api {
    pub fn run() {}
}
```

Nếu module cha có nhiều tầng, mọi tầng cần public hoặc cần re-export:

```rust
mod internal {
    pub mod api {
        pub fn run() {}
    }
}

pub use internal::api::run;
```

External code gọi:

```rust
my_library::run();
```

### 17.4. `cannot find type in this scope`

Sai:

```rust
fn make() -> User {
    User::new()
}
```

nhưng `User` nằm ở module khác và chưa import. Sửa:

```rust
use crate::domain::User;
```

hoặc dùng full path:

```rust
fn make() -> crate::domain::User {
    crate::domain::User::new()
}
```

### 17.5. Hai item cùng tên

Sai:

```rust
use std::fmt::Result;
use std::io::Result;
```

Sửa bằng module parent:

```rust
use std::{fmt, io};

fn write_value() -> fmt::Result {
    Ok(())
}

fn read_value() -> io::Result<()> {
    Ok(())
}
```

Hoặc alias:

```rust
use std::fmt::Result;
use std::io::Result as IoResult;
```

### 17.6. Đã import trait nhưng vẫn gọi method lỗi

Kiểm tra trait có thực sự trong scope không:

```rust
use std::io::Read;
```

Nếu dùng method qua trait tự định nghĩa từ module khác, import trait:

```rust
use crate::formatting::PrettyPrint;
```

### 17.7. Dùng `crate::` sai trong `main.rs`

Nếu logic nằm ở `src/lib.rs`:

```rust
// Sai hoặc không đúng mục đích trong src/main.rs:
use crate::domain::User;

// Đúng:
use package_name::domain::User;
```

`crate` luôn có nghĩa “crate hiện tại”; binary và library trong cùng package là hai crate khác nhau.

### 17.8. Có cả `foo.rs` và `foo/mod.rs`

Sai cấu trúc:

```text
src/foo.rs
src/foo/mod.rs
```

Chọn một kiểu:

```text
src/foo.rs
src/foo/bar.rs
```

hoặc:

```text
src/foo/mod.rs
src/foo/bar.rs
```

### 17.9. Quên `pub` khi tái xuất

Sai:

```rust
mod domain;
use domain::User;
```

External crate không nhìn thấy `User`. Sửa:

```rust
mod domain;
pub use domain::User;
```

### 17.10. Tên package và tên crate không giống

Kiểm tra:

```toml
[package]
name = "my-package"

[lib]
name = "my_core"
```

Trong source dùng:

```rust
use my_core::SomeType;
```

Không nên đoán tên; hãy xem `[lib] name`, tên dependency bên trái trong `Cargo.toml` và thông báo lỗi của Cargo.

---

## 18. Quy trình tìm lỗi path có hệ thống

Khi một path không biên dịch, kiểm tra lần lượt từ trái sang phải:

| Bước | Câu hỏi |
|---|---|
| 1 | Tôi đang ở crate nào: binary, library, test, example hay dependency? |
| 2 | Path nên bắt đầu bằng `crate`, `self`, `super`, tên dependency hay tên module tương đối? |
| 3 | Module đã được khai báo bằng `mod` chưa? |
| 4 | File module có đúng vị trí và đúng tên không? |
| 5 | Mỗi module trên path đã có visibility phù hợp chưa? |
| 6 | Item cuối cùng có `pub` hoặc visibility phù hợp chưa? |
| 7 | Nếu là type/method từ trait, trait đã được `use` chưa? |
| 8 | Nếu là dependency, `Cargo.toml` có đúng tên, feature, version và path không? |
| 9 | Nếu là library cùng package, có đang dùng package/library crate name thay vì `crate::` không? |
| 10 | Cargo có đang build đúng package/feature/target không? |

Các lệnh hữu ích:

```bash
cargo check
cargo check --all-targets
cargo check --all-features
cargo tree
cargo metadata --no-deps
cargo test --workspace
rustc --explain E0432
rustc --explain E0433
rustc --explain E0603
```

`cargo tree` cho biết dependency graph. `cargo metadata` cho biết package, target và dependency mà Cargo đang nhận diện. `rustc --explain CODE` giải thích chi tiết một mã lỗi compiler.

---

## 19. Quy tắc thiết kế module nên áp dụng

**Thứ nhất, dùng module để biểu diễn ranh giới trách nhiệm**, không chỉ để chia file. Một module tốt thường có một chủ đề rõ ràng như `domain`, `parser`, `storage` hoặc `http`.

**Thứ hai, mặc định để module private.** Chỉ công khai những gì thật sự là API. `pub(crate)` thường phù hợp hơn `pub` cho helper dùng chung nội bộ.

**Thứ ba, giữ public API ngắn và ổn định bằng re-export.** Người dùng nên có thể viết:

```rust
use my_library::{Client, Config, Error};
```

thay vì phụ thuộc vào đường dẫn triển khai sâu.

**Thứ tư, phân biệt module declaration với import.** `mod` quyết định compiler đưa mã nào vào crate; `use` chỉ tạo shortcut; `pub use` tạo shortcut public.

**Thứ năm, không dùng glob import trong public module nếu không có lý do rõ ràng.** Glob có thể khiến tên trong scope thay đổi khi dependency nâng cấp.

**Thứ sáu, đưa logic vào library crate khi package có binary.** Binary nên gọi library qua tên crate. Cách này làm API được kiểm tra như một client thật và giúp unit/integration test rõ ràng hơn.

**Thứ bảy, dùng `crate::` cho các liên kết nội bộ ổn định.** Dùng `super::` khi module con và module cha có quan hệ chặt và nên di chuyển cùng nhau. Tránh path tương đối dài xuyên qua nhiều module không liên quan.

**Thứ tám, không biến workspace thành một module tree khổng lồ.** Mỗi package là một crate boundary riêng; muốn gọi package khác phải khai báo dependency và chỉ dùng public API của crate đó.

---

## 20. Bảng ghi nhớ nhanh

```rust
// Khai báo module inline
mod api {
    pub fn run() {}
}

// Khai báo module từ file api.rs hoặc api/mod.rs
mod api;

// Gọi module hiện tại / path tương đối
api::run();

// Path tuyệt đối trong cùng crate
crate::api::run();

// Module hiện tại
self::run();

// Module cha
super::run();

// Tạo shortcut private trong scope
use crate::api::run;

// Alias
use crate::api::run as start;

// Nested import
use std::{fs, io::{self, Read}};

// Glob import
use crate::api::*;

// Re-export public
pub use crate::api::run;

// Chỉ public trong crate
pub(crate) fn internal() {}

// Chỉ public với module cha
pub(super) fn parent_only() {}

// Public trong một module ancestor cụ thể
pub(in crate::application) fn app_only() {}

// Gọi library cùng package từ binary
use package_name::some_public_item;

// Gọi dependency ngoài
use dependency_name::SomeType;
```

---

---

## 21. Namespace và cơ chế name resolution

### 21.1. Vì sao cùng một tên đôi khi không xung đột?

Rust không đặt mọi tên vào một không gian duy nhất. Compiler phân loại tên theo **namespace** rồi phân giải tên dựa trên ngữ cảnh sử dụng. Rust Reference mô tả các namespace chính gồm type namespace, value namespace, macro namespace, lifetime namespace và label namespace. [10]

| Namespace | Thành phần thường gặp | Ngữ cảnh sử dụng |
|---|---|---|
| **Type** | Module, struct, enum, trait, type alias, type parameter | `let x: User`, `impl Trait for Type` |
| **Value** | Function, constant, static, constructor struct/enum, local variable | `run()`, `User(1)`, `Status::Ready` |
| **Macro** | `macro_rules!`, derive macro, attribute macro, function-like proc macro | `vec![...]`, `#[derive(...)]`, `custom!(...)` |
| **Lifetime** | `'a`, `'static` | `&'a str` |
| **Label** | `'loop_name` | `break 'loop_name` |

Ví dụ một struct tuple có tên trong cả type namespace và value namespace:

```rust
struct User(u64);

fn main() {
    let user: User = User(10);
    println!("{}", user.0);
}
```

Trong `let user: User`, `User` là tên kiểu. Trong `User(10)`, `User` là constructor nằm trong value namespace. Tên macro cũng có thể trùng với tên type ở namespace khác:

```rust
struct Token;

macro_rules! Token {
    () => {
        "macro Token"
    };
}

fn main() {
    let _value: Token = Token;
    println!("{}", Token!());
}
```

Không nên cố tình đặt tên trùng nếu không có lý do, nhưng hiểu namespace giúp giải thích vì sao một số mã hợp lệ trong khi tên giống nhau.

### 21.2. Name resolution xảy ra theo ngữ cảnh

Cùng một chuỗi `Item` có thể được tìm trong type namespace hoặc value namespace:

```rust
struct Item {
    id: u64,
}

fn make_item() -> Item {
    Item { id: 1 }
}

fn main() {
    let value: Item = make_item();
    println!("{}", value.id);
}
```

Trong type position, `Item` là struct type. Trong expression position, `Item { ... }` là struct constructor. Với associated item, compiler còn phải xét inherent implementation và các trait implementation.

### 21.3. Tên import có thể chạm nhiều namespace

Một `use` không phải lúc nào cũng chỉ tạo một binding đơn giản. Nó có thể đưa type, value hoặc macro vào namespace tương ứng:

```rust
use std::fmt::Result;
use std::io::Result as IoResult;

fn format_value() -> Result {
    Ok(())
}

fn read_value() -> IoResult<()> {
    Ok(())
}
```

Nếu hai `use` đưa cùng một tên vào cùng một namespace, compiler thường báo `E0252`. Hãy dùng alias hoặc giữ module cha trong path.

### 21.4. Canonical path và alias path

Mỗi item được định nghĩa trong module có một canonical path trong crate, chẳng hạn:

```rust
mod domain {
    pub struct User;
}
```

Canonical path của type là:

```text
crate::domain::User
```

Nếu re-export:

```rust
pub use domain::User;
```

thì `crate::User` là một alias path đến cùng item, không phải một type mới. Vì vậy:

```rust
crate::domain::User
crate::User
```

vẫn là cùng một type. Hai package khác nhau không có một global canonical namespace chung; canonical path chỉ có ý nghĩa trong crate định nghĩa item. [9]

---

## 22. Toàn bộ dạng path nâng cao

Rust Reference định nghĩa path là chuỗi path segment nối bằng `::`; path có thể dùng để chỉ item, value, type, macro hoặc attribute. [9]

### 22.1. `crate`, `self`, `super`, `Self` và `$crate`

| Từ khóa | Ý nghĩa | Vị trí thường dùng |
|---|---|---|
| `crate` | Crate hiện tại | `crate::domain::User` |
| `self` | Module hiện tại; trong method, `self` còn là receiver | `self::helper()`, `self.field` |
| `super` | Module cha | `super::Config` |
| `Self` | Kiểu đang được định nghĩa hoặc implement | `fn new() -> Self`, `Self::CONST` |
| `$crate` | Crate định nghĩa `macro_rules!` macro | Trong macro expansion |
| `::` | Global path; ý nghĩa phụ thuộc edition | `::std::fmt::Display` |

Ví dụ sử dụng `Self`:

```rust
trait Factory {
    type Output;

    fn create() -> Self;
    fn output(&self) -> Self::Output;
}

struct User {
    id: u64,
}

impl Factory for User {
    type Output = u64;

    fn create() -> Self {
        Self { id: 1 }
    }

    fn output(&self) -> Self::Output {
        self.id
    }
}
```

`Self` giúp code không lặp tên kiểu và giữ đúng khi kiểu được đổi tên.

### 22.2. `::` và khác biệt edition

Trong Edition 2018 trở lên, `crate::` là cách rõ ràng để bắt đầu từ root crate hiện tại. Path bắt đầu bằng `::` được phân giải từ extern prelude và thường phải tiếp theo bằng tên crate:

```rust
let now = ::std::time::Instant::now();
```

Trong code Rust hiện đại, thường viết:

```rust
let now = std::time::Instant::now();
```

Đối với code nội bộ:

```rust
crate::domain::User
```

Đối với dependency:

```rust
serde::Serialize
```

Không nên mang quy tắc path của Edition 2015 sang Edition 2024 một cách máy móc. Nếu đang nâng edition, hãy đọc thông báo compiler và dùng `cargo fix --edition` khi phù hợp. Quy tắc global path theo edition được mô tả trong Rust Reference. [9]

### 22.3. Turbofish `::<...>`

`::` cũng được dùng để truyền generic arguments một cách tường minh:

```rust
let values = (0..5).collect::<Vec<_>>();
let bytes = Vec::<u8>::with_capacity(16);
```

Với function generic:

```rust
fn identity<T>(value: T) -> T {
    value
}

fn main() {
    let value = identity::<u64>(10);
    println!("{value}");
}
```

Trong nhiều trường hợp compiler suy luận được kiểu:

```rust
let value = identity(10_u64);
```

Dùng turbofish khi type inference không đủ thông tin hoặc khi muốn làm rõ API.

### 22.4. Associated type và fully qualified path

```rust
trait Repository {
    type Item;

    fn load(&self) -> Self::Item;
}

fn use_item<R>(repository: &R) -> R::Item
where
    R: Repository,
{
    repository.load()
}
```

Khi có nhiều trait cùng tên associated type hoặc method, dùng fully qualified syntax:

```rust
trait A {
    type Output;
}

trait B {
    type Output;
}

struct Item;

impl A for Item {
    type Output = u32;
}

impl B for Item {
    type Output = String;
}

fn main() {
    let _: <Item as A>::Output = 1;
    let _: <Item as B>::Output = String::from("text");
}
```

Với method trùng tên:

```rust
trait Printable {
    fn print(&self);
}

trait Loggable {
    fn print(&self);
}

struct Message;

impl Printable for Message {
    fn print(&self) {
        println!("printable");
    }
}

impl Loggable for Message {
    fn print(&self) {
        println!("loggable");
    }
}

fn main() {
    let message = Message;
    <Message as Printable>::print(&message);
    <Message as Loggable>::print(&message);
}
```

### 22.5. Path trong type, expression, pattern và macro

Path có thể xuất hiện trong nhiều vị trí:

```rust
use std::collections::HashMap;

fn build_map() -> HashMap<String, u64> {
    HashMap::new()
}

fn classify(value: Result<u64, String>) {
    match value {
        std::result::Result::Ok(number) => println!("{number}"),
        std::result::Result::Err(error) => eprintln!("{error}"),
    }
}
```

Trong pattern có thể rút gọn:

```rust
enum State {
    Ready,
    Failed,
}

use State::*;

fn handle(state: State) {
    match state {
        Ready => println!("ready"),
        Failed => println!("failed"),
    }
}
```

Glob trong pattern giúp mã ngắn nhưng có thể làm người đọc khó biết variant thuộc enum nào. Trong API lớn, viết `State::Ready` thường rõ hơn.

---

## 23. `use` chuyên sâu và các biến thể visibility

### 23.1. `use` không kéo theo child module

Đoạn sau chỉ đưa module `api` vào scope, không đưa toàn bộ item con thành tên ngắn:

```rust
use crate::service::api;

fn run() {
    api::start();
}
```

Nếu muốn gọi trực tiếp `start()`, phải import thêm:

```rust
use crate::service::api::start;
```

Hoặc dùng nested path:

```rust
use crate::service::api::{self, start};
```

### 23.2. Import trait mà không tạo tên bằng `as _`

Khi chỉ cần trait để method resolution nhưng không muốn thêm tên trait vào scope:

```rust
use std::fmt::Write as _;

fn main() {
    let mut output = String::new();
    write!(&mut output, "id={}", 10).unwrap();
    println!("{output}");
}
```

Mẫu này nói rõ rằng trait được import để kích hoạt method/macro behavior, không phải để sử dụng trực tiếp tên `Write`.

### 23.3. `pub(crate) use`, `pub(super) use` và `pub(in ...) use`

Visibility áp dụng được cho `use`:

```rust
mod internal {
    pub(crate) struct Cache;
}

pub(crate) use internal::Cache;
```

`Cache` có thể được dùng qua `crate::Cache` trong toàn crate nhưng không trở thành public API cho dependency bên ngoài.

```rust
mod outer {
    mod inner {
        pub(super) struct Token;
    }

    use inner::Token;

    pub fn make_token() -> Token {
        Token
    }
}
```

`pub use` phải tuân thủ visibility chain. Không thể public re-export một item mà bên ngoài không thể truy cập qua bất kỳ đường hợp lệ nào, trừ khi re-export cắt ngắn được private chain theo đúng quy tắc Rust.

### 23.4. Import có điều kiện

```rust
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
fn mode(metadata: &std::fs::Metadata) -> u32 {
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn mode(_: &std::fs::Metadata) -> u32 {
    0
}
```

Cần đặt `#[cfg]` đồng bộ lên cả `use` và item sử dụng. Nếu chỉ điều kiện hóa `use` nhưng function vẫn được biên dịch trên platform không có trait, sẽ phát sinh lỗi.

### 23.5. Tránh `use super::*` trong production

Trong unit test, `use super::*` tiện để truy cập item của module cha. Trong module production, nên import tên cụ thể:

```rust
use crate::domain::{Order, User};
```

thay vì:

```rust
use super::*;
```

Import cụ thể làm dependency của module hiển thị rõ, giảm nguy cơ tên mới được thêm vào module cha làm phát sinh xung đột.

### 23.6. Xử lý tên trùng giữa module và type

Nếu có module và type cùng khái niệm, có thể giữ namespace riêng nhưng nên đặt tên file/module rõ:

```rust
mod user;

use user::User;
```

Nếu tên local bị khó đọc:

```rust
use crate::user::User as DomainUser;
use external_api::User as ApiUser;
```

Alias nên thể hiện vai trò, không chỉ thêm hậu tố tùy tiện:

```rust
use crate::error::Error as AppError;
use std::io::Error as IoError;
```

---

## 24. Cargo target: mỗi target là một crate riêng

Cargo package gồm các target có thể biên dịch thành crate: library, binary, example, test và benchmark. Cargo thường suy luận target từ cấu trúc thư mục, nhưng có thể cấu hình chi tiết trong manifest. [11]

| Target | File mặc định | Cách chạy/kiểm tra | Quyền truy cập |
|---|---|---|---|
| Library | `src/lib.rs` | `cargo build`, `cargo test`, `cargo doc` | Được binary/example/test link vào |
| Binary chính | `src/main.rs` | `cargo run` | Dùng public API của library cùng package |
| Binary phụ | `src/bin/name.rs` | `cargo run --bin name` | Dùng public API library |
| Example | `examples/name.rs` | `cargo run --example name` | Giống external client, chỉ public API |
| Integration test | `tests/name.rs` | `cargo test --test name` | Giống external crate, chỉ public API |
| Benchmark | `benches/name.rs` | `cargo bench --bench name` | Thường dùng public API |
| Proc macro | crate `proc-macro` | Được dependency khác dùng | Không dùng trong chính crate định nghĩa |

### 24.1. Cấu hình library target

```toml
[lib]
name = "my_core"
path = "src/lib.rs"
crate-type = ["rlib", "cdylib"]
```

`name` là tên library crate mà package khác import. Nếu không đặt, Cargo dùng tên package và đổi `-` thành `_`. Một package chỉ có tối đa một library target. [11]

Các crate type quan trọng:

| `crate-type` | Mục đích |
|---|---|
| `lib` | Library phụ thuộc theo cách compiler lựa chọn |
| `rlib` | Rust library để crate Rust khác link |
| `dylib` | Dynamic Rust library |
| `cdylib` | Dynamic library cho FFI C/C++/ngôn ngữ khác |
| `staticlib` | Static library |
| `proc-macro` | Procedural macro crate |

### 24.2. Nhiều binary target

```text
src/
├── lib.rs
├── main.rs
└── bin/
    ├── import_users.rs
    └── export_users.rs
```

`Cargo.toml` có thể cấu hình rõ:

```toml
[[bin]]
name = "import-users"
path = "src/bin/import_users.rs"
required-features = ["import"]

[[bin]]
name = "export-users"
path = "src/bin/export_users.rs"
required-features = ["export"]
```

Chạy:

```bash
cargo run --bin import-users --features import
cargo run --bin export-users --features export
```

Nếu target có `required-features` nhưng feature chưa bật, Cargo bỏ qua target đó thay vì build. [11]

### 24.3. Example target

```toml
[[example]]
name = "basic"
path = "examples/basic.rs"

[[example]]
name = "postgres"
path = "examples/postgres.rs"
required-features = ["postgres"]
```

```bash
cargo run --example basic
cargo run --example postgres --features postgres
cargo build --examples
```

Example là client rất tốt để kiểm tra API có thực sự thuận tiện không. Nếu example phải truy cập module private, đó thường là dấu hiệu public API chưa được thiết kế đúng hoặc example nên trở thành unit test.

### 24.4. Integration test là crate riêng

Mỗi file trực tiếp dưới `tests/` thường được biên dịch thành một integration test executable riêng:

```text
tests/
├── public_api.rs
├── error_cases.rs
└── common/
    └── mod.rs
```

`tests/common/mod.rs` là module hỗ trợ, không phải một test target độc lập nếu đặt dưới thư mục con.

```rust
// tests/common/mod.rs
pub fn sample_name() -> &'static str {
    "An"
}
```

```rust
// tests/public_api.rs
mod common;

use my_library::User;

#[test]
fn creates_user() {
    let user = User::new(common::sample_name());
    assert_eq!(user.name(), "An");
}
```

### 24.5. Tắt auto-discovery khi tên module xung đột

Nếu library muốn có module `src/bin`, Cargo có thể nhầm các file trong `src/bin` là binary targets. Có thể tắt:

```toml
[package]
name = "my-library"
version = "0.1.0"
edition = "2024"
autobins = false
```

Chỉ dùng các cờ `autolib`, `autobins`, `autoexamples`, `autotests`, `autobenches` khi cấu trúc đặc biệt thực sự cần thiết.

---

## 25. Cargo dependency đầy đủ hơn

### 25.1. Các dạng khai báo dependency

```toml
[dependencies]
# Registry mặc định
serde = "1"

# Registry + feature
serde = { version = "1", features = ["derive"] }

# Git branch
parser = { git = "https://github.com/example/parser", branch = "next" }

# Git tag
parser = { git = "https://github.com/example/parser", tag = "v2.0.0" }

# Git revision
parser = { git = "https://github.com/example/parser", rev = "abcdef123456" }

# Local path
shared = { path = "../shared" }

# Path khi phát triển, version khi publish
shared = { path = "../shared", version = "1.0" }

# Đổi tên dependency bên trái
json = { package = "serde_json", version = "1" }

# Optional dependency
image = { version = "0.25", optional = true }

# Tắt default feature của dependency
regex = { version = "1", default-features = false }
```

Cargo phân biệt location của dependency với version requirement. `path` hoặc `git` được dùng trong môi trường local; nếu khai báo đồng thời `version`, Cargo có thể dùng registry version khi publish và kiểm tra bản local khớp version requirement. [4]

### 25.2. Dev dependency và build dependency không tự truyền tiếp

```toml
[dev-dependencies]
assert_cmd = "2"

[build-dependencies]
cc = "1"
```

`assert_cmd` dùng cho test/example/benchmark; `cc` dùng cho `build.rs`. Source library không thể tự động dùng `cc` chỉ vì `build.rs` có dependency đó. Nếu cả source và build script đều cần, khai báo ở cả hai section.

### 25.3. Registry khác

```toml
[dependencies]
internal-types = {
    version = "2",
    registry = "company-registry",
}
```

Tên registry được cấu hình trong `.cargo/config.toml` hoặc cấu hình Cargo tương ứng. Không nên đưa thông tin credentials vào `Cargo.toml` hoặc commit token vào repository.

### 25.4. `package` dùng để rename dependency

```toml
[dependencies]
http_types = { package = "http", version = "1" }
```

Trong Rust:

```rust
use http_types::StatusCode;
```

Tên `http_types` ở bên trái là tên crate trong source; `http` ở `package` là package thực tế trên registry. Điều này hữu ích khi package name có dấu gạch ngang, muốn dùng tên domain rõ hơn hoặc cần đồng thời tham chiếu hai nguồn khác nhau.

### 25.5. Patch dependency trong quá trình phát triển

Root `Cargo.toml` có thể thay dependency registry bằng bản local/Git:

```toml
[patch.crates-io]
my-library = { path = "../my-library" }
```

Sau đó mọi dependency trong graph trỏ tới `my-library` từ crates.io có thể được thay bằng bản local phù hợp version. `patch` chỉ nên đặt ở workspace root và cần kiểm tra bằng:

```bash
cargo tree
cargo update
```

Khi publish, `patch` không được gửi như một phần dependency resolution của người dùng; nó là override phía project đang build.

### 25.6. Chọn dependency với `default-features = false`

```toml
[dependencies]
serde = { version = "1", default-features = false, features = ["alloc"] }
```

Tuy nhiên, nếu một dependency khác trong graph bật default features của `serde`, việc tắt ở một dòng chưa chắc làm toàn graph tắt. Hãy kiểm tra feature graph:

```bash
cargo tree -e features
cargo tree -e features -i serde
```

Cargo feature thường được hợp nhất theo package trong dependency graph. Features nên thiết kế additive, tránh feature này vô hiệu hóa behavior của feature khác. [12]

### 25.7. `Cargo.lock`, reproducible build và offline

Trong application/binary, thường nên commit `Cargo.lock` để build CI và production tái lập. Với library publish lên registry, chính sách commit lockfile có thể khác; hãy thống nhất theo project.

Các lệnh kiểm soát build:

```bash
cargo fetch
cargo build --locked
cargo test --locked
cargo build --offline
cargo build --frozen
```

`--locked` yêu cầu Cargo không thay đổi lockfile. `--frozen` tương đương vừa locked vừa offline. Dùng trong CI để phát hiện dependency resolution không nhất quán.

---

## 26. Feature và conditional compilation toàn diện

Cargo feature là cơ chế bật/tắt conditional compilation và optional dependency. Feature được định nghĩa trong `[features]`, được bật bằng `--features`, `--all-features` hoặc thông qua dependency declaration. [12]

### 26.1. Feature cơ bản

```toml
[features]
default = ["json"]
json = []
xml = []
full = ["json", "xml"]
```

```rust
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "xml")]
pub mod xml;
```

Chạy các cấu hình:

```bash
cargo check
cargo check --no-default-features
cargo check --features xml
cargo check --features "json xml"
 cargo check --all-features
```

### 26.2. Optional dependency được gom qua feature domain

```toml
[dependencies]
serde = { version = "1", optional = true, features = ["derive"] }
serde_json = { version = "1", optional = true }

[features]
default = []
serialization = ["dep:serde", "dep:serde_json"]
```

`dep:` ngăn Cargo tự tạo feature public tên `serde` hoặc `serde_json`; người dùng chỉ thấy feature domain `serialization`.

```rust
#[cfg(feature = "serialization")]
pub mod serialization {
    pub fn encode<T: serde::Serialize>(value: &T) -> serde_json::Result<String> {
        serde_json::to_string(value)
    }
}
```

### 26.3. `cfg`, `cfg_attr`, `all`, `any`, `not`

```rust
#[cfg(all(unix, feature = "native"))]
mod unix_native;

#[cfg(any(target_os = "linux", target_os = "android"))]
fn is_linux_family() -> bool {
    true
}

#[cfg(not(feature = "std"))]
fn fallback() {}
```

Thêm attribute có điều kiện:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub port: u16,
}
```

### 26.4. Kiểm tra target và feature

```bash
rustc --print=cfg
rustc --print=cfg --target x86_64-pc-windows-msvc
cargo check --target x86_64-unknown-linux-gnu
```

Trong source:

```rust
#[cfg(target_os = "windows")]
const CONFIG_FILE: &str = "config\\app.toml";

#[cfg(not(target_os = "windows"))]
const CONFIG_FILE: &str = "config/app.toml";
```

### 26.5. Feature mutually exclusive

Tốt nhất thiết kế feature additive. Nếu thật sự không thể bật đồng thời:

```rust
#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("Bật chỉ một trong `sqlite` hoặc `postgres`");
```

Nhưng thường nên thiết kế một abstraction cho phép hai backend cùng compile, rồi chọn backend lúc runtime hoặc tách thành package riêng.

### 26.6. Feature trong workspace

```bash
cargo check --workspace --all-features
cargo check -p app --features app/json
cargo check -p app -p core --features app/json,core/serde
```

Với resolver hiện đại, chỉ rõ `package/feature` giúp tránh bật nhầm feature của member khác.

---

## 27. Build script và module sinh tự động

Build script `build.rs` được Cargo compile rồi chạy trước khi build package. Nó thường dùng để compile C, tìm native library, tạo source Rust hoặc phát hiện cấu hình hệ thống. [13]

### 27.1. Cấu trúc cơ bản

```text
my-package/
├── Cargo.toml
├── build.rs
└── src/
    └── lib.rs
```

`Cargo.toml`:

```toml
[package]
name = "my-package"
version = "0.1.0"
edition = "2024"

[build-dependencies]
```

`build.rs`:

```rust
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=schema.txt");
}
```

Cú pháp `cargo::KEY=VALUE` được Cargo hỗ trợ ở các phiên bản hiện đại; khi cần tương thích Cargo cũ có thể dùng `cargo:KEY=VALUE`. [13]

### 27.2. Sinh file vào `OUT_DIR`

`build.rs`:

```rust
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=schemas/version.txt");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let version = fs::read_to_string("schemas/version.txt")
        .expect("cannot read schemas/version.txt");

    let generated = format!(
        "pub const SCHEMA_VERSION: &str = {:?};\\n",
        version.trim()
    );

    fs::write(out_dir.join("generated.rs"), generated)
        .expect("cannot write generated.rs");
}
```

`src/lib.rs`:

```rust
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub use generated::SCHEMA_VERSION;
```

Build script nên ghi file sinh vào `OUT_DIR`, không ghi đè trực tiếp vào `src/`. Cargo yêu cầu build script không dựa vào việc `OUT_DIR` luôn rỗng vì nội dung có thể tồn tại giữa các build. [13]

### 27.3. `rerun-if-changed` và `rerun-if-env-changed`

```rust
fn main() {
    println!("cargo::rerun-if-changed=schemas/api.json");
    println!("cargo::rerun-if-env-changed=API_MODE");
}
```

Nếu không khai báo các điều kiện rerun, Cargo có thể dùng chiến lược bảo thủ và chạy lại script khi bất kỳ file nào trong package thay đổi. Khai báo input cụ thể giúp build nhanh và dễ dự đoán hơn. [13]

### 27.4. Truyền biến compile-time

`build.rs`:

```rust
fn main() {
    println!("cargo::rustc-env=APP_BUILD_MODE=production");
}
```

`src/lib.rs`:

```rust
pub const BUILD_MODE: &str = env!("APP_BUILD_MODE");
```

Không nên dùng `env!` để phụ thuộc vào một biến runtime chỉ có khi chạy Cargo; hãy phân biệt compile-time metadata với runtime configuration.

### 27.5. Custom `cfg` từ build script

`build.rs`:

```rust
fn main() {
    println!("cargo::rustc-check-cfg=cfg(has_schema)");

    if std::path::Path::new("schema.sql").exists() {
        println!("cargo::rustc-cfg=has_schema");
    }
}
```

`src/lib.rs`:

```rust
#[cfg(has_schema)]
pub mod database_schema;
```

`cfg` do build script tạo ra không tự động bật Cargo feature hoặc optional dependency; nó chỉ thêm cờ compile-time cho rustc. [13]

### 27.6. Cross-compilation: host và target

`build.rs` chạy trên **host**, trong khi source package được compile cho **target**. Không nên dùng `cfg!` trong build script để đoán target platform; hãy đọc các biến `CARGO_CFG_*` mà Cargo truyền vào build script. [13]

---

## 28. Macro và module: cách khai báo, gọi, export

### 28.1. `macro_rules!` cơ bản

```rust
macro_rules! say {
    ($message:expr) => {
        println!("{}", $message);
    };
}

fn main() {
    say!("hello");
}
```

Macro có thể sinh expression, statement, item, type hoặc pattern. Macro by example gồm matcher và transcriber; các fragment specifier thường gặp là `expr`, `ident`, `item`, `path`, `stmt`, `ty`, `pat`, `tt` và `vis`. [14]

### 28.2. Repetition `*`, `+`, `?`

```rust
macro_rules! make_vec {
    ($($value:expr),* $(,)?) => {{
        let mut result = Vec::new();
        $(result.push($value);)*
        result
    }};
}

fn main() {
    let values = make_vec![1, 2, 3];
    println!("{values:?}");
}
```

Ý nghĩa:

| Toán tử | Ý nghĩa |
|---|---|
| `*` | Không hoặc nhiều lần |
| `+` | Ít nhất một lần |
| `?` | Không hoặc một lần |

### 28.3. Macro trong module và thứ tự khai báo

`macro_rules!` có cơ chế textual scope khác item thông thường. Macro thường phải được khai báo trước khi gọi theo textual scope, trừ khi được export/import theo path. [14]

```rust
mod macros {
    macro_rules! internal_log {
        ($value:expr) => {
            println!("log: {}", $value);
        };
    }

    pub(crate) use internal_log;
}

fn main() {
    macros::internal_log!("hello");
}
```

`pub(crate) use` biến macro local thành path-based binding trong phạm vi crate mà không buộc `#[macro_export]` đưa macro ra root public API.

### 28.4. `#[macro_export]` và `$crate`

```rust
#[doc(hidden)]
pub fn __announce_impl(message: &str) {
    println!("ANNOUNCE: {message}");
}

#[macro_export]
macro_rules! announce {
    ($message:expr) => {
        $crate::__announce_impl($message)
    };
}
```

`$crate` trỏ tới crate định nghĩa macro, không phải crate gọi macro. Điều này quan trọng khi macro được dùng từ dependency khác; nếu viết `crate::__announce_impl`, `crate` có thể trỏ sai crate ở call site. Rust Reference mô tả `$crate` là path tới top-level crate nơi macro được định nghĩa. [9] [14]

### 28.5. Macro export và public helper

Nếu macro public mở rộng thành path gọi helper, helper phải có thể truy cập từ crate bên ngoài. Có thể dùng `#[doc(hidden)] pub` để helper public về mặt kỹ thuật nhưng ẩn khỏi tài liệu chính:

```rust
#[doc(hidden)]
pub fn __private_helper() {}
```

Không nên dùng `pub(crate)` cho helper mà macro public ở crate khác cần gọi, vì macro expansion sẽ không truy cập được helper private.

### 28.6. Procedural macro

Procedural macro có ba loại: function-like, derive và attribute macro. Chúng phải nằm trong crate có `proc-macro = true` và không thể được sử dụng từ chính crate định nghĩa; một crate khác phải import chúng. [15]

Cấu trúc workspace tối thiểu:

```text
macro-workspace/
├── Cargo.toml
├── app/
│   ├── Cargo.toml
│   └── src/main.rs
└── app-derive/
    ├── Cargo.toml
    └── src/lib.rs
```

`app-derive/Cargo.toml`:

```toml
[package]
name = "app-derive"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true
```

`app-derive/src/lib.rs`:

```rust
use proc_macro::TokenStream;

#[proc_macro_derive(Describe)]
pub fn describe(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let name = source
        .split_whitespace()
        .nth(1)
        .expect("expected a named item");

    format!(
        "impl Describe for {name} {{ fn describe(&self) -> &'static str {{ \"{name}\" }} }}"
    )
    .parse()
    .unwrap()
}
```

`app/src/main.rs`:

```rust
use app_derive::Describe;

trait Describe {
    fn describe(&self) -> &'static str;
}

#[derive(Describe)]
struct User;

fn main() {
    println!("{}", User.describe());
}
```

Ví dụ trên chỉ nhằm minh họa cơ chế. Proc macro production nên dùng parser/generator chuyên dụng, tạo error span tốt và tránh phân tích token bằng `split_whitespace`.

### 28.7. Hygiene và path trong proc macro

Procedural macro là unhygienic theo Rust Reference, vì token output hoạt động gần như code được viết trực tiếp tại call site. Macro author cần dùng path đầy đủ hoặc tên helper ít khả năng xung đột. [15]

---

## 29. Test, example, doctest và benchmark

Cargo có unit test, integration test, documentation test và benchmark target. Unit test trong target có thể truy cập private API; integration test trong `tests/` chỉ dùng public API; example cũng dùng public API library. [11] [16]

### 29.1. Unit test private item

```rust
fn normalize(input: &str) -> String {
    input.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn trims_and_lowercases() {
        assert_eq!(normalize("  RUST  "), "rust");
    }
}
```

### 29.2. Integration test public API

```rust
// tests/api.rs
use my_library::normalize_public;

#[test]
fn public_contract_works() {
    assert_eq!(normalize_public(" RUST "), "rust");
}
```

Không import private `crate::...` trong integration test. Integration test là một crate độc lập được link với library.

### 29.3. Documentation test

```rust
/// Returns the sum of two numbers.
///
/// ```
/// use my_library::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

Cargo test sẽ compile và chạy code block trong documentation của library theo cơ chế rustdoc. [16]

Nếu code block chỉ để minh họa không compile:

```rust
/// ```text
/// this is output, not Rust code
/// ```
```

Nếu cần ẩn một dòng setup khỏi rendered documentation nhưng vẫn compile, dùng dòng bắt đầu bằng `#`:

```rust
/// ```
/// # let value = 10;
/// assert_eq!(value, 10);
/// ```
```

### 29.4. Test có `Result`

```rust
#[test]
fn parses_value() -> Result<(), Box<dyn std::error::Error>> {
    let value: u64 = "42".parse()?;
    assert_eq!(value, 42);
    Ok(())
}
```

### 29.5. Chọn test target và truyền argument

```bash
cargo test
cargo test normalize
cargo test --lib
cargo test --test api
cargo test --doc
cargo test --all-targets
cargo test --all-features
cargo test -- --nocapture
cargo test -- --test-threads=1
cargo test --no-run
cargo test --no-fail-fast
```

Tham số trước `--` thuộc Cargo; tham số sau `--` được truyền cho test harness. [16]

### 29.6. Kiểm tra binary từ integration test

```rust
#[test]
fn cli_prints_version() {
    let binary = env!("CARGO_BIN_EXE_my-cli");
    let output = std::process::Command::new(binary)
        .arg("--version")
        .output()
        .expect("failed to run binary");

    assert!(output.status.success());
}
```

Tên biến dùng dấu gạch dưới theo quy tắc Cargo: `CARGO_BIN_EXE_<name>`, trong đó `<name>` là binary target.

### 29.7. Benchmark target

```text
benches/
└── parsing.rs
```

Trên stable, benchmark thường dùng crate như Criterion thay vì phụ thuộc vào `#[bench]` nightly. Nếu dùng benchmark target:

```bash
cargo bench
cargo bench --bench parsing
```

---

## 30. Rustdoc và thiết kế public API

### 30.1. Module docs và crate docs

Trong `src/lib.rs`:

```rust
//! # My Library
//!
//! Library cung cấp API xử lý user.

#![warn(missing_docs)]

/// Tạo một user mới.
pub fn create_user() {}
```

`//!` là documentation cho module/crate hiện tại. `///` là documentation cho item ngay sau nó.

### 30.2. Intra-doc links

```rust
/// Tạo [`User`] từ [`Config`].
///
/// Xem thêm [`crate::parse_user`] và [`std::result::Result`].
pub fn create_user() {}
```

Dùng path trong doc link giúp rustdoc kiểm tra liên kết trong quá trình `cargo doc`/`cargo test`.

### 30.3. Re-export và documentation

Nếu implementation nằm ở module private:

```rust
mod internal;

#[doc(inline)]
pub use internal::Client;
```

`#[doc(inline)]` có thể làm tài liệu của `Client` xuất hiện tại vị trí re-export. Không nên lạm dụng; public API quan trọng nên có docs ở nơi người dùng nhìn thấy.

### 30.4. Kiểm tra tài liệu

```bash
cargo doc --no-deps --open
cargo doc --workspace --all-features
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Mục tiêu là mọi item public có tài liệu đủ để người dùng biết import path, ownership, lỗi trả về, feature yêu cầu và ví dụ sử dụng.

### 30.5. Public API và private implementation

Mẫu facade:

```rust
mod domain;
mod error;
mod storage;

pub use domain::{Order, User};
pub use error::Error;
pub use storage::Client;
```

Người dùng chỉ phụ thuộc vào:

```rust
use my_library::{Client, Error, User};
```

Khi refactor file nội bộ, path public vẫn giữ nguyên. Đây là lý do `pub use` không chỉ rút ngắn cú pháp mà còn là công cụ quản lý tương thích API.

---

## 31. Mẫu thiết kế module thực tế

### 31.1. Facade module

```text
src/
├── lib.rs
├── domain.rs
├── error.rs
└── infrastructure.rs
```

```rust
// lib.rs
mod domain;
mod error;
mod infrastructure;

pub use domain::{Order, User};
pub use error::Error;
pub use infrastructure::Client;
```

Phù hợp cho library muốn công khai API ngắn, giấu implementation.

### 31.2. Layered modules

```text
src/
├── lib.rs
├── domain/
│   ├── mod.rs hoặc domain.rs
│   ├── user.rs
│   └── order.rs
├── application/
│   ├── user_service.rs
│   └── order_service.rs
└── infrastructure/
    ├── database.rs
    └── http.rs
```

`domain` không nên phụ thuộc ngược vào `infrastructure`. `application` có thể phụ thuộc `domain`; `infrastructure` triển khai trait do `domain` hoặc `application` định nghĩa.

### 31.3. Sealed trait để giới hạn implementation bên ngoài

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait PublicTrait: sealed::Sealed {
    fn run(&self);
}

pub struct BuiltIn;

impl sealed::Sealed for BuiltIn {}

impl PublicTrait for BuiltIn {
    fn run(&self) {
        println!("built-in");
    }
}
```

Vì trait `Sealed` không public, crate bên ngoài không thể implement `PublicTrait` cho kiểu của họ. Mẫu này giúp library giữ quyền kiểm soát tập implementation và thay đổi trait an toàn hơn.

### 31.4. Type-state module

```rust
pub mod connection {
    pub struct Disconnected;
    pub struct Connected;

    pub struct Connection<State> {
        state: State,
    }

    impl Connection<Disconnected> {
        pub fn new() -> Self {
            Self { state: Disconnected }
        }

        pub fn connect(self) -> Connection<Connected> {
            Connection { state: Connected }
        }
    }

    impl Connection<Connected> {
        pub fn send(&self, message: &str) {
            println!("send: {message}");
        }
    }
}
```

Module public chỉ expose các chuyển trạng thái hợp lệ; compiler ngăn gọi `send` trước khi `connect`.

### 31.5. Platform module

```rust
#[cfg(unix)]
mod platform;

#[cfg(windows)]
mod platform;

pub use platform::current_username;
```

Mỗi platform có file triển khai cùng public contract:

```text
src/
├── lib.rs
├── platform.rs              # có thể dùng cfg nội bộ
└── platform/
    ├── unix.rs
    └── windows.rs
```

Hoặc:

```rust
#[cfg(unix)]
#[path = "platform/unix.rs"]
mod platform;

#[cfg(windows)]
#[path = "platform/windows.rs"]
mod platform;
```

### 31.6. `no_std` và module

Library có thể không dùng standard library:

```rust
#![no_std]

pub mod math {
    pub fn saturating_add(left: u32, right: u32) -> u32 {
        left.saturating_add(right)
    }
}
```

Nếu cần hỗ trợ `std` tùy chọn:

```toml
[features]
default = ["std"]
std = []
```

```rust
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod io_support;
```

Thiết kế `no_std` cần kiểm tra mọi dependency và feature; không chỉ module của mình là đủ.

---

## 32. Các lỗi thường gặp nâng cao

| Mã/lỗi | Nguyên nhân thường gặp | Cách xử lý |
|---|---|---|
| `E0432 unresolved import` | Sai path, thiếu `mod`, thiếu `use` | Kiểm tra cây module và import cụ thể |
| `E0433 failed to resolve` | Tên module/crate không tồn tại trong scope | Kiểm tra `Cargo.toml`, `mod`, tên alias |
| `E0603 private` | Module/item/field không đủ visibility | Mở đúng ancestor hoặc dùng re-export |
| `E0252 name defined multiple times` | Hai import cùng tên cùng namespace | Alias hoặc giữ parent module |
| `E0412 cannot find type` | Type chưa import hoặc sai namespace | `use crate::...::Type` |
| `E0425 cannot find value` | Function/constant/variable chưa trong scope | Kiểm tra value namespace |
| `E0599 no method named` | Trait chưa import hoặc kiểu không implement trait | `use Trait as _;`, kiểm tra bound |
| `E0034 multiple applicable items` | Nhiều trait có method cùng tên | `<Type as Trait>::method(...)` |
| `E0463 can't find crate` | Dependency/target/toolchain không có | Kiểm tra manifest, target và toolchain |
| `duplicate module` | Có cả `foo.rs` và `foo/mod.rs` hoặc khai báo `mod foo` hai lần | Chọn một file và một declaration |
| `unresolved extern crate` | Dùng tên package thay vì tên dependency alias | Kiểm tra khóa bên trái `[dependencies]` |
| `cannot find macro` | Macro chưa export/import hoặc sai textual scope | `use crate::macro_name;`, `#[macro_export]` hoặc `pub use` |
| `proc-macro crate types currently cannot export` | Proc-macro crate export item runtime không hợp lệ | Tách runtime library và proc-macro package |
| `feature ... not found` | Gọi feature chưa khai báo hoặc sai tên package | Kiểm tra `[features]`, `cargo tree -e features` |

### 32.1. Chẩn đoán `E0432`

Giả sử:

```text
src/
├── lib.rs
└── domain/
    └── user.rs
```

Nếu `lib.rs` viết:

```rust
use crate::domain::user::User;
```

nhưng thiếu:

```rust
mod domain;
```

thì compiler không biết `domain` tồn tại. Cần thêm:

```rust
mod domain;
```

và `domain` phải khai báo:

```rust
pub mod user;
```

nếu đường dẫn đó được dùng từ một module bên ngoài `domain`.

### 32.2. Chẩn đoán `E0603`

Phân tích path từ trái sang phải:

```text
external_crate::a::b::Item
```

Kiểm tra `a`, sau đó `b`, cuối cùng `Item`. Mỗi đoạn phải truy cập được từ vị trí gọi. Chỉ thêm `pub` cho `Item` không đủ nếu `a` hoặc `b` private.

### 32.3. Chẩn đoán `E0599`

Kiểm tra theo thứ tự:

```rust
value.method()
```

1. `method` có phải inherent method không?
2. Nếu là trait method, trait đã import chưa?
3. Kiểu của `value` có implement trait không?
4. Trait implementation có bị `#[cfg]` loại khỏi build không?
5. Có generic bound nào thiếu không?
6. Có hai method trùng tên cần UFCS không?

Ví dụ:

```rust
use std::io::Read;

fn read(mut input: impl Read) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;
    Ok(buffer)
}
```

### 32.4. Chẩn đoán lỗi macro

Nếu macro không tìm thấy helper:

```rust
#[macro_export]
macro_rules! make_value {
    () => {
        helper()
    };
}
```

khi gọi từ crate khác, `helper()` được tìm tại call site và có thể thất bại. Sửa bằng `$crate`:

```rust
#[doc(hidden)]
pub fn helper() -> u32 {
    42
}

#[macro_export]
macro_rules! make_value {
    () => {
        $crate::helper()
    };
}
```

### 32.5. Chẩn đoán Cargo build sai target

Nếu code đúng nhưng Cargo không build file mới, kiểm tra:

```bash
cargo metadata --no-deps --format-version 1
cargo tree
cargo check --all-targets
cargo check --manifest-path path/to/Cargo.toml
```

Có thể bạn đang đứng ở workspace root, đang chạy package khác, target có `required-features`, hoặc file nằm trong thư mục Cargo không auto-discover.

---

## 33. Cookbook: cách gọi theo từng tình huống

| Tình huống | Cách viết nên dùng |
|---|---|
| Function cùng module | `run()` hoặc `self::run()` |
| Module con gọi module cha | `super::helper()` |
| Module bất kỳ trong cùng crate | `crate::domain::User` |
| Import type | `use crate::domain::User;` |
| Import nhiều item cùng parent | `use crate::domain::{Order, User};` |
| Import cả module và item | `use crate::domain::{self, User};` |
| Import trait chỉ để method | `use crate::Trait as _;` |
| Public facade | `pub use crate::domain::User;` |
| Nội bộ toàn crate | `pub(crate) use ...` |
| Binary gọi library cùng package | `use package_name::PublicType;` |
| Example gọi library | `use package_name::PublicType;` |
| Integration test gọi library | `use package_name::PublicType;` |
| Workspace package gọi package khác | khai báo `path` dependency rồi `use dependency_name::...` |
| Feature module | `#[cfg(feature = "x")] mod x;` |
| Platform module | `#[cfg(unix)] mod platform;` |
| Macro public gọi helper | `$crate::__helper()` |
| Trait method trùng tên | `<Type as Trait>::method(...)` |
| Associated type trùng tên | `<Type as Trait>::Output` |
| Generic ambiguity | `Vec::<u8>::new()` |
| Build-generated module | `include!(concat!(env!("OUT_DIR"), "/generated.rs"));` |

### 33.1. Gọi sibling module

```rust
// src/lib.rs
mod parser;
mod validator;

pub fn process(input: &str) -> bool {
    let value = crate::parser::parse(input);
    crate::validator::validate(value)
}
```

### 33.2. Gọi child module

```rust
mod service {
    pub mod user {
        pub fn create() {}
    }

    pub fn start() {
        user::create();
    }
}
```

### 33.3. Gọi parent module từ child

```rust
fn config_value() -> u64 {
    10
}

mod worker {
    pub fn run() -> u64 {
        super::config_value()
    }
}
```

### 33.4. Binary gọi library

```rust
// src/lib.rs
pub mod api {
    pub fn run() {}
}
```

```rust
// src/main.rs
use package_name::api;

fn main() {
    api::run();
}
```

### 33.5. External client gọi re-export

```rust
// library
mod internal;
pub use internal::Client;
```

```rust
// client
use library_name::Client;
```

### 33.6. Gọi dependency bị alias

```toml
[dependencies]
json = { package = "serde_json", version = "1" }
```

```rust
use json::Value;
```

### 33.7. Gọi API feature-gated

```rust
#[cfg(feature = "sqlite")]
use crate::storage::SqliteStore;

#[cfg(feature = "sqlite")]
pub fn create_store() -> SqliteStore {
    SqliteStore::new()
}
```

Caller phải bật feature:

```bash
cargo add my-library --features sqlite
```

### 33.8. Gọi trait extension

```rust
trait Normalize {
    fn normalized(&self) -> String;
}

impl Normalize for str {
    fn normalized(&self) -> String {
        self.trim().to_lowercase()
    }
}

fn main() {
    use Normalize as _;
    println!("{}", " Rust ".normalized());
}
```

---

## 34. Một project mẫu đầy đủ từ đầu đến cuối

Mục tiêu là tạo package có library, binary, module nội bộ, re-export, feature, example, integration test và nhiều target.

### 34.1. Tạo project

```bash
cargo new catalog_app
cd catalog_app
mkdir -p src/domain examples tests
```

Cấu trúc:

```text
catalog_app/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── error.rs
│   ├── domain.rs
│   └── domain/
│       ├── product.rs
│       └── category.rs
├── examples/
│   └── list_products.rs
└── tests/
    └── public_api.rs
```

### 34.2. Manifest

```toml
[package]
name = "catalog_app"
version = "0.1.0"
edition = "2024"
autobins = false

[features]
default = []
json = []

[lib]
name = "catalog_core"
path = "src/lib.rs"

[[bin]]
name = "catalog-app"
path = "src/main.rs"
```

### 34.3. Error module

```rust
// src/error.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    EmptyName,
    NotFound,
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "name cannot be empty"),
            Self::NotFound => write!(formatter, "product not found"),
        }
    }
}

impl std::error::Error for CatalogError {}
```

### 34.4. Domain modules

```rust
// src/domain.rs
mod category;
mod product;

pub use category::Category;
pub use product::Product;
```

```rust
// src/domain/category.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Book,
    Tool,
    Other,
}
```

```rust
// src/domain/product.rs
use super::Category;
use crate::error::CatalogError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    name: String,
    category: Category,
}

impl Product {
    pub fn new(name: impl Into<String>, category: Category) -> Result<Self, CatalogError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CatalogError::EmptyName);
        }

        Ok(Self { name, category })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> Category {
        self.category
    }
}
```

### 34.5. Library root và facade API

```rust
// src/lib.rs
mod domain;
mod error;

pub use domain::{Category, Product};
pub use error::CatalogError;

pub fn sample_product() -> Result<Product, CatalogError> {
    Product::new("Rust Book", Category::Book)
}
```

### 34.6. Binary gọi library

```rust
// src/main.rs
use catalog_core::sample_product;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let product = sample_product()?;
    println!("{} ({:?})", product.name(), product.category());
    Ok(())
}
```

### 34.7. Example gọi public API

```rust
// examples/list_products.rs
use catalog_core::{Category, Product};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product::new("Programming Tool", Category::Tool)?;
    println!("{}", product.name());
    Ok(())
}
```

### 34.8. Integration test

```rust
// tests/public_api.rs
use catalog_core::{Category, Product};

#[test]
fn product_is_created_through_public_api() {
    let product = Product::new("Rust", Category::Book).unwrap();
    assert_eq!(product.name(), "Rust");
}
```

### 34.9. Chạy toàn bộ

```bash
cargo check
cargo build
cargo run --bin catalog-app
cargo run --example list_products
cargo test
cargo test --all-targets --all-features
cargo doc --no-deps
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Project này minh họa các boundary quan trọng: `domain` private, `Product` public qua re-export, binary/example/test là client bên ngoài library, còn `crate::error` được dùng nội bộ từ library.

---

## 35. Bộ lệnh Cargo cần biết

| Mục đích | Lệnh |
|---|---|
| Tạo binary package | `cargo new app` |
| Tạo library package | `cargo new --lib library` |
| Khởi tạo trong thư mục hiện tại | `cargo init` |
| Kiểm tra nhanh, không link executable | `cargo check` |
| Build debug | `cargo build` |
| Build release | `cargo build --release` |
| Chạy binary mặc định | `cargo run` |
| Chạy binary cụ thể | `cargo run --bin name` |
| Chạy example | `cargo run --example name` |
| Chạy test | `cargo test` |
| Chạy doc test | `cargo test --doc` |
| Sinh docs | `cargo doc --open` |
| Format | `cargo fmt` |
| Lint | `cargo clippy` |
| Xem dependency | `cargo tree` |
| Xem feature graph | `cargo tree -e features` |
| Xem metadata | `cargo metadata --no-deps` |
| Thêm dependency | `cargo add serde` |
| Xóa dependency | `cargo remove serde` |
| Cập nhật dependency | `cargo update` |
| Tải dependency trước | `cargo fetch` |
| Kiểm tra package có thể publish | `cargo package` |
| Xem file sẽ được package | `cargo package --list` |
| Kiểm tra workspace | `cargo check --workspace` |
| Build package cụ thể | `cargo build -p package_name` |
| Giải thích mã lỗi compiler | `rustc --explain E0603` |
| Liệt kê target cfg | `rustc --print=cfg` |
| Liệt kê target triples | `rustc --print target-list` |

### 35.1. Quy trình kiểm tra chuẩn trước commit

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```

Nếu project hỗ trợ nhiều feature quan trọng, thêm các lần chạy:

```bash
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo check --workspace --features "feature_a feature_b"
```

---

## 36. Checklist khi tạo hoặc tách module

Trước khi tạo module mới, hãy trả lời các câu hỏi sau:

1. Module này thuộc crate nào: binary, library, example, test hay proc-macro?
2. Nó nên inline, nằm ở `foo.rs`, hay ở `foo/mod.rs`?
3. Module cha đã có `mod foo;` chưa?
4. Module có cần public không, hay chỉ cần `pub(crate)`?
5. Item bên trong cần visibility nào?
6. Caller nằm cùng module, child module, sibling module, crate khác hay package khác?
7. Path nên bắt đầu bằng `crate`, `self`, `super` hay tên dependency?
8. Có nên re-export item ở root để giữ public API ngắn không?
9. Module có feature/platform condition không?
10. Có unit test private và integration test public tương ứng chưa?
11. Nếu là library, đã viết rustdoc và example chưa?
12. Nếu là workspace member, đã khai báo dependency rõ ràng chưa?

### 36.1. Checklist khi gặp lỗi gọi module

```text
[ ] Đúng thư mục project/Cargo.toml chưa?
[ ] Đúng package/target đang build chưa?
[ ] Có `mod module_name;` ở module cha chưa?
[ ] Tên file có khớp snake_case không?
[ ] Có trùng `foo.rs` và `foo/mod.rs` không?
[ ] Path bắt đầu đúng bằng crate/self/super/dependency chưa?
[ ] Các ancestor module đã public hoặc có re-export chưa?
[ ] Item cuối đã public chưa?
[ ] Trait cần thiết đã import chưa?
[ ] Dependency có trong đúng section của Cargo.toml chưa?
[ ] Feature có bật chưa?
[ ] Target platform có làm item bị cfg loại bỏ không?
[ ] Có cần `cargo clean` sau thay đổi build script không?
[ ] Đã chạy `cargo metadata`, `cargo tree`, `rustc --explain` chưa?
```

---

## 37. Tóm tắt phân biệt các câu lệnh dễ nhầm

| Câu lệnh | Compiler/Cargo hiểu là |
|---|---|
| `mod foo;` | Module `foo` được định nghĩa trong file hoặc inline; đưa vào crate tree |
| `use crate::foo::Bar;` | Tạo binding tên `Bar` trong scope hiện tại |
| `pub use crate::foo::Bar;` | Tạo binding và public re-export |
| `pub mod foo;` | Module `foo` public theo visibility chain |
| `[dependencies] foo = "1"` | Cargo thêm package/crate dependency vào target cần nó |
| `extern crate foo;` | Khai báo external crate kiểu cũ/đặc biệt |
| `include!(...)` | Chèn token source vào scope hiện tại |
| `#[path = "..."] mod foo;` | Module `foo` lấy source từ path chỉ định |
| `#[cfg(...)] mod foo;` | Chỉ khai báo/compile module khi điều kiện đúng |
| `pub(crate)` | Public trong crate hiện tại, không public với dependency |
| `pub(in crate::x)` | Public trong module `x` và descendants của nó |

---

## 38. Kết luận mở rộng

Một dự án Rust được tổ chức tốt không chỉ có các file `.rs`; nó có một hệ thống boundary rõ ràng. **Package** là đơn vị Cargo quản lý. **Target** là nguồn tạo ra các crate. **Crate** là boundary compiler và dependency. **Module** là cây namespace bên trong crate. **Path** chỉ ra item. **`use`** tạo shortcut. **Visibility** quyết định ai được phép dùng. **`pub use`** thiết kế API mà người dùng nhìn thấy.

Khi code không gọi được một item, đừng chỉ thêm `pub` một cách ngẫu nhiên. Hãy xác định đúng vị trí caller, xác định crate hiện tại, vẽ path từ root đến item, kiểm tra từng ancestor, rồi chọn giữa full path, `use`, `pub use`, `pub(crate)` hoặc re-architecture.

Công thức thực hành đầy đủ là:

> **Đúng package → đúng target → đúng crate → đúng `mod` declaration → đúng namespace → đúng path qualifier → đúng visibility → đúng `use`/re-export → đúng dependency/feature → đúng test target.**

Nếu xây dựng library, hãy giữ implementation private và public API ổn định ở crate root. Nếu xây dựng application, hãy đặt logic dùng chung trong library và để binary chỉ làm nhiệm vụ khởi động. Nếu xây dựng workspace, hãy chia boundary theo domain hoặc trách nhiệm độc lập, không coi workspace như một module duy nhất. Nếu dùng macro hoặc code generation, hãy kiểm soát scope, `$crate`, `OUT_DIR`, feature và cross-compilation ngay từ đầu.

---

## 39. Lập trình mạng trong Rust: bản đồ tổng thể

Rust có thể làm việc với mạng ở nhiều tầng. Ở tầng thấp, `std::net` cung cấp TCP và UDP đồng bộ. Ở tầng bất đồng bộ, `tokio::net` cung cấp các kiểu tương đương như `TcpListener`, `TcpStream` và `UdpSocket`. Ở tầng giao thức, `reqwest` phù hợp cho HTTP client, `axum` phù hợp cho HTTP server, `rumqttc` cho MQTT và `tokio-tungstenite` cho WebSocket. Tokio cung cấp runtime, task, timer, channel và các primitive cần thiết để chạy nhiều thao tác I/O đồng thời. [17] [18] [19] [20] [21] [22]

| Nhu cầu | Giao thức/thư viện nên dùng | Kiểu giao tiếp | Ví dụ thực tế |
|---|---|---|---|
| Gọi REST/JSON API | `reqwest` + `serde` | Client request/response | Gọi dịch vụ thanh toán, lấy dữ liệu |
| Tạo REST API | `axum` + `tokio` | Server HTTP | Backend, microservice |
| Kết nối liên tục có thứ tự | TCP + Tokio | Byte stream | Chat, proxy, giao thức riêng |
| Gói tin độc lập, ít overhead | UDP + Tokio | Datagram | Telemetry, discovery, game |
| Kết nối hai chiều trên HTTP | WebSocket | Message stream | Chat, realtime dashboard |
| IoT publish/subscribe | MQTT + `rumqttc` | Topic/message broker | Cảm biến, thiết bị |
| Mã hóa đường truyền | TLS/Rustls | Lớp bảo mật | HTTPS, MQTTS, WSS |

Một nguyên tắc quan trọng là **không dùng cùng một abstraction cho mọi giao thức**. HTTP có request/response và status code; TCP chỉ là byte stream không có ranh giới message; UDP giữ ranh giới datagram nhưng có thể mất hoặc đảo thứ tự gói; WebSocket có frame/message sau khi handshake HTTP; MQTT có broker, topic, QoS và session. Nếu không phân biệt các đặc tính này, code có thể biên dịch nhưng hoạt động sai khi mạng chậm hoặc bị ngắt.

### 39.1. Manifest mẫu cho các ví dụ mạng

Các chương sau dùng các crate sau:

```toml
[package]
name = "rust-network-demo"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
futures-util = "0.3"
reqwest = { version = "0.13", default-features = false, features = ["json", "query", "rustls"] }
rumqttc = "0.25"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.30"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

`reqwest` cần feature `json` nếu dùng `.json(...)`, feature `query` nếu dùng `.query(...)`, và một backend TLS như `rustls` nếu gọi HTTPS với manifest đã tắt default features. Reqwest có cả async client và blocking client; async client cần runtime Tokio. Khi dùng nhiều request, nên tạo một `Client` và tái sử dụng nó để hưởng lợi từ connection pooling. [17]

### 39.2. Cấu trúc project mạng nên dùng

```text
rust-network-demo/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── error.rs
│   ├── config.rs
│   ├── http_api.rs
│   ├── mqtt.rs
│   ├── tcp.rs
│   ├── udp.rs
│   └── websocket.rs
├── examples/
│   ├── http_client.rs
│   ├── mqtt_client.rs
│   ├── tcp_server.rs
│   ├── tcp_client.rs
│   ├── udp_demo.rs
│   └── websocket_client.rs
└── tests/
    └── api_contract.rs
```

Mỗi file giao thức là một module. `lib.rs` nên re-export các API ổn định; `examples/` là các chương trình client/server độc lập. Cấu trúc này kết nối trực tiếp với các quy tắc `mod`, `use`, `pub` và `pub use` đã trình bày ở các chương trước.

---

## 40. Async Rust với Tokio: nền tảng bắt buộc

### 40.1. Runtime và `#[tokio::main]`

Một `async fn` chỉ tạo ra future; nó chưa tự chạy. Runtime phải poll future đó. Cách đơn giản nhất là bật macro Tokio:

```rust
#[tokio::main]
async fn main() {
    println!("Tokio runtime is running");
}
```

Tương đương về ý tưởng:

```rust
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        println!("running inside Tokio");
    });
}
```

`#[tokio::main]` cần feature macro và runtime phù hợp; dùng `features = ["full"]` là cách thuận tiện cho ví dụ học tập. Tokio hướng tới các workload I/O-bound và cung cấp runtime đa luồng, async I/O, task, timer, channel và networking. Với công việc CPU-bound, nên cân nhắc `spawn_blocking`, Rayon hoặc một worker thread riêng thay vì chiếm executor thread bằng vòng lặp tính toán dài. [22]

### 40.2. `await` không tạo thread mới

```rust
async fn one_request() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://example.com").await?;
    response.text().await
}
```

`await` tạm dừng future khi thao tác I/O chưa sẵn sàng và nhường quyền cho task khác. Nó không đồng nghĩa với tạo một OS thread mới.

### 40.3. Tuần tự và đồng thời

Tuần tự:

```rust
let first = reqwest::get("https://example.com/one").await?;
let second = reqwest::get("https://example.com/two").await?;
```

Hai request được khởi động đồng thời:

```rust
let first = reqwest::get("https://example.com/one");
let second = reqwest::get("https://example.com/two");

let (first, second) = tokio::join!(first, second);
let first = first?.text().await?;
let second = second?.text().await?;
```

`join!` chờ tất cả future hoàn tất. Nếu cần thu thập một danh sách lớn, dùng `futures_util::future::join_all`; nếu cần giới hạn số tác vụ đồng thời, dùng `Semaphore` thay vì spawn vô hạn.

### 40.4. `tokio::spawn` và điều kiện `'static`

```rust
let task = tokio::spawn(async move {
    println!("background task");
});

task.await?;
```

Future đưa vào `tokio::spawn` thường phải sở hữu dữ liệu và thỏa `'static`, bởi task có thể sống lâu hơn stack frame đã tạo nó. Nếu cần chia sẻ dữ liệu, dùng `Arc`; nếu cần thay đổi dữ liệu giữa các task, kết hợp `Arc<Mutex<T>>` hoặc channel.

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let counter = Arc::new(Mutex::new(0_u64));
let worker_counter = Arc::clone(&counter);

tokio::spawn(async move {
    let mut value = worker_counter.lock().await;
    *value += 1;
});
```

Không giữ `std::sync::MutexGuard` qua `.await`. Nếu lock cần được giữ trong async code, ưu tiên `tokio::sync::Mutex`, hoặc lấy dữ liệu ra khỏi lock trước khi await để giảm contention.

### 40.5. `select!` và graceful shutdown

```rust
use tokio::signal;
use tokio::time::{sleep, Duration};

async fn worker() {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(5)) => {
                println!("periodic work");
            }
            result = signal::ctrl_c() => {
                if let Err(error) = result {
                    eprintln!("signal error: {error}");
                }
                println!("shutdown requested");
                break;
            }
        }
    }
}
```

Trong service thật, nên tạo một shutdown signal dùng chung cho HTTP server, MQTT event loop, TCP accept loop và các background task. Mỗi task cần thoát khi nhận signal thay vì bị process kill đột ngột.

### 40.6. Timeout và cancellation

```rust
use std::time::Duration;
use tokio::time::timeout;

let result = timeout(Duration::from_secs(3), async {
    reqwest::get("https://example.com").await
}).await;

match result {
    Ok(Ok(response)) => println!("status = {}", response.status()),
    Ok(Err(error)) => eprintln!("request error: {error}"),
    Err(_) => eprintln!("request timed out"),
}
```

`tokio::time::timeout` trả về `Err(Elapsed)` nếu future không hoàn tất trong thời gian chỉ định; khi timeout bị drop, future bên trong bị hủy theo cơ chế cancellation của future. Timeout chỉ có tác dụng khi future nhường quyền; một đoạn code CPU-bound không yield có thể vượt thời gian mà không bị ngắt ngay. [25]

---

## 41. HTTP/REST API client với Reqwest

### 41.1. GET đơn giản

```rust
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let response = reqwest::get("https://httpbin.org/get").await?;
    println!("status: {}", response.status());
    println!("body: {}", response.text().await?);
    Ok(())
}
```

`response.text().await?` tiêu thụ body và trả về `String`. Nếu cần đọc bytes, dùng `.bytes().await?`; nếu muốn deserialize JSON typed, dùng `.json::<T>().await?`.

### 41.2. Tái sử dụng `Client`, header và query

```rust
use reqwest::{header, Client};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::builder()
        .user_agent("rust-network-demo/0.1")
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client
        .get("https://httpbin.org/get")
        .query(&[("page", "1"), ("limit", "20")])
        .header(header::ACCEPT, "application/json")
        .send()
        .await?;

    println!("{}", response.url());
    println!("{}", response.text().await?);
    Ok(())
}
```

`Client` nên được tạo một lần và truyền vào service bằng `Arc<Client>` nếu nhiều task cùng sử dụng. Không nên tạo client mới bên trong mỗi lần gọi API vì làm mất lợi ích của connection pool và có thể tạo quá nhiều connection.

### 41.3. Kiểm tra HTTP status đúng cách

Lỗi transport và HTTP error là hai loại khác nhau. DNS failure, timeout, TLS failure hoặc connection reset là lỗi transport. HTTP `404`, `401` hoặc `500` là response hợp lệ về mặt transport nhưng biểu thị lỗi nghiệp vụ/protocol.

```rust
let response = client.get(url).send().await?;
let status = response.status();

if !status.is_success() {
    let body = response.text().await.unwrap_or_default();
    return Err(format!("HTTP {}: {}", status, body).into());
}

let payload = response.json::<ApiResponse>().await?;
```

Có thể dùng `.error_for_status()?` để chuyển status 4xx/5xx thành `reqwest::Error`:

```rust
let payload = client
    .get(url)
    .send()
    .await?
    .error_for_status()?
    .json::<ApiResponse>()
    .await?;
```

Trong production, nên giữ lại status code và response body có giới hạn kích thước để chẩn đoán, nhưng không log token, password hoặc dữ liệu nhạy cảm.

### 41.4. Deserialize JSON typed bằng Serde

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UserResponse {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct CreateUserRequest<'a> {
    name: &'a str,
    email: &'a str,
}
```

Đọc JSON:

```rust
let user: UserResponse = client
    .get("https://api.example.com/users/1")
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
```

Gửi JSON:

```rust
let request = CreateUserRequest {
    name: "An",
    email: "an@example.com",
};

let created: UserResponse = client
    .post("https://api.example.com/users")
    .json(&request)
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
```

Serde JSON hỗ trợ `Value` cho dữ liệu động, nhưng với API ổn định nên ưu tiên `Serialize`/`Deserialize` typed để compiler và IDE kiểm tra field. `serde_json::from_str`, `from_slice`, `from_reader`, `to_string`, `to_vec` và macro `json!` là các API thường dùng. [23]

### 41.5. JSON động với `serde_json::Value`

```rust
use serde_json::{json, Value};

let body = json!({
    "event": "created",
    "data": {
        "id": 42,
        "active": true
    }
});

let active = body["data"]["active"].as_bool().unwrap_or(false);
let raw: Value = client
    .post("https://httpbin.org/post")
    .json(&body)
    .send()
    .await?
    .json()
    .await?;

println!("active={active}, response={raw}");
```

Không nên dùng indexing `value["field"]` cho dữ liệu bắt buộc mà không kiểm tra vì field không tồn tại có thể cho `Value::Null`. Với contract quan trọng, hãy deserialize vào struct và dùng `Option<T>` cho field thực sự optional.

### 41.6. POST form và raw body

```rust
let form = [("username", "an"), ("password", "not-a-real-password")];
let response = client
    .post("https://httpbin.org/post")
    .form(&form)
    .send()
    .await?;
```

Raw body:

```rust
let response = client
    .post("https://httpbin.org/post")
    .header(reqwest::header::CONTENT_TYPE, "text/plain")
    .body("hello from Rust")
    .send()
    .await?;
```

### 41.7. PUT, PATCH và DELETE

```rust
let update = serde_json::json!({ "name": "An updated" });

client
    .put("https://api.example.com/users/1")
    .json(&update)
    .send()
    .await?
    .error_for_status()?;

client
    .patch("https://api.example.com/users/1")
    .json(&update)
    .send()
    .await?
    .error_for_status()?;

client
    .delete("https://api.example.com/users/1")
    .send()
    .await?
    .error_for_status()?;
```

### 41.8. Bearer token, API key và Basic Auth

```rust
let response = client
    .get("https://api.example.com/profile")
    .bearer_auth(token)
    .send()
    .await?;
```

API key trong header:

```rust
let response = client
    .get("https://api.example.com/data")
    .header("X-API-Key", api_key)
    .send()
    .await?;
```

Basic Auth:

```rust
let response = client
    .get("https://api.example.com/private")
    .basic_auth(username, Some(password))
    .send()
    .await?;
```

Không hard-code secret trong source, test hoặc Git. Đọc secret từ environment/secret manager và không in toàn bộ request headers trong log.

### 41.9. Retry có backoff và điều kiện an toàn

Không phải request nào cũng được retry. GET thường idempotent hơn POST; POST chỉ nên retry khi API hỗ trợ idempotency key hoặc contract bảo đảm không tạo bản ghi trùng.

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn get_with_retry(
    client: &reqwest::Client,
    url: &str,
    attempts: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut delay = Duration::from_millis(200);

    for attempt in 0..attempts {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response)
                if response.status().is_server_error() && attempt + 1 < attempts => {}
            Ok(response) => return response.error_for_status(),
            Err(error) if attempt + 1 < attempts => {
                eprintln!("attempt {} failed: {error}", attempt + 1);
            }
            Err(error) => return Err(error),
        }

        sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(5));
    }

    unreachable!("attempts must be greater than zero")
}
```

Trong production nên thêm jitter ngẫu nhiên, tôn trọng `Retry-After`, giới hạn tổng thời gian, phân biệt lỗi timeout với lỗi validation và dùng circuit breaker khi downstream liên tục thất bại.

### 41.10. Pagination

```rust
#[derive(Debug, Deserialize)]
struct Page<T> {
    items: Vec<T>,
    next_cursor: Option<String>,
}

async fn load_all_users(client: &reqwest::Client) -> Result<Vec<UserResponse>, reqwest::Error> {
    let mut result = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut request = client.get("https://api.example.com/users");
        if let Some(value) = &cursor {
            request = request.query(&[("cursor", value)]);
        }

        let page: Page<UserResponse> = request
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        result.extend(page.items);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    Ok(result)
}
```

### 41.11. Blocking client

Nếu chương trình chỉ gọi một vài request và không cần async, dùng feature `blocking`:

```toml
reqwest = { version = "0.13", features = ["blocking", "json", "rustls"] }
```

```rust
fn main() -> Result<(), reqwest::Error> {
    let response = reqwest::blocking::get("https://example.com")?;
    println!("{}", response.text()?);
    Ok(())
}
```

Không dùng `reqwest::blocking` bên trong async runtime, vì API blocking có thể chặn worker thread. Nếu buộc phải gọi thư viện blocking trong async code, dùng `tokio::task::spawn_blocking`.

---

## 42. Xây REST API server bằng Axum

Axum tập trung vào routing, handler, extractor và response; nó tích hợp với Tokio và dùng hệ sinh thái Tower/Tower HTTP cho middleware như timeout, tracing, compression và authorization. [21]

### 42.1. Server tối thiểu

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, Rust" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### 42.2. Tách route thành module

```rust
// src/api/mod.rs
use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> &'static str {
    "ok"
}
```

```rust
// src/main.rs
mod api;

use axum::Router;

#[tokio::main]
async fn main() {
    let app = Router::new().merge(api::router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

`api` có thể private vì `main.rs` là crate root của binary. Nếu muốn dùng router từ integration test hoặc library crate khác, chuyển router vào `src/lib.rs` và re-export function public.

### 42.3. Path, Query và JSON extractor

```rust
use axum::{
    extract::{Path, Query},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    page: Option<u32>,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: u64,
    name: String,
}

async fn get_user(Path(id): Path<u64>) -> Json<UserResponse> {
    Json(UserResponse {
        id,
        name: "An".to_owned(),
    })
}

async fn search(Query(params): Query<SearchParams>) -> Json<HashMap<&'static str, String>> {
    let mut result = HashMap::new();
    result.insert("query", params.q.unwrap_or_default());
    result.insert("page", params.page.unwrap_or(1).to_string());
    Json(result)
}

fn router() -> Router {
    Router::new()
        .route("/users/{id}", get(get_user))
        .route("/search", get(search))
}
```

Axum 0.8 dùng syntax path như `/users/{id}`. `Json<T>` ở request cần `Deserialize`; `Json<T>` ở response cần `Serialize`. Extractor body như `Json` thường đặt cuối danh sách handler arguments vì nó tiêu thụ request body.

### 42.4. POST JSON và status code

```rust
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Debug, Serialize)]
struct CreatedUser {
    id: u64,
    name: String,
}

async fn create_user(Json(input): Json<CreateUser>) -> impl IntoResponse {
    let output = CreatedUser { id: 1, name: input.name };
    (StatusCode::CREATED, Json(output))
}
```

### 42.5. Chia sẻ state bằng `Arc` và `State`

```rust
use axum::{extract::State, routing::get, Router};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
}

async fn upstream(State(state): State<Arc<AppState>>) -> String {
    match state.client.get("https://example.com").send().await {
        Ok(response) => format!("upstream status: {}", response.status()),
        Err(error) => format!("upstream error: {error}"),
    }
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/upstream", get(upstream))
        .with_state(state)
}
```

Axum clone state cho mỗi request; bọc state lớn trong `Arc` giúp clone rẻ. Một `reqwest::Client` có thể nằm trong state để mọi handler dùng chung connection pool. [21]

### 42.6. Error type chuyển thành HTTP response

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
enum ApiError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("upstream request failed")]
    Upstream(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::Upstream(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

async fn handler() -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({ "ok": true })))
}
```

Không trả stack trace hoặc `reqwest::Error` chi tiết cho client production. Hãy log chi tiết ở server với request ID, còn response chỉ trả mã lỗi và thông điệp an toàn.

### 42.7. Middleware và timeout

Axum dùng Tower middleware. Ví dụ timeout cho toàn router:

```rust
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

let app = Router::new()
    .route("/", get(root))
    .layer(TimeoutLayer::new(Duration::from_secs(10)));
```

Thêm dependency:

```toml
tower-http = { version = "0.6", features = ["timeout", "trace"] }
```

Khi triển khai production, nên có timeout ở nhiều lớp: timeout kết nối downstream, timeout request của server, timeout đọc body và timeout graceful shutdown. Một timeout tổng thể không thay thế cho mọi timeout chi tiết.

### 42.8. Graceful shutdown cho Axum

```rust
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// axum::serve(listener, app)
//     .with_graceful_shutdown(shutdown_signal())
//     .await
//     .unwrap();
```

---

## 43. TCP socket: stream, framing và server nhiều client

### 43.1. TCP không có message boundary

TCP đảm bảo một byte stream có thứ tự, không bảo đảm mỗi lần `write` tương ứng với một lần `read`. Nếu client gửi hai message, server có thể đọc chúng thành một lần hoặc nhiều lần. Vì vậy, ứng dụng phải định nghĩa **framing protocol**:

| Kiểu framing | Cách xác định kết thúc | Khi dùng |
|---|---|---|
| Newline-delimited | `\n` | Text command, log, demo |
| Length-prefixed | Header chứa độ dài | Binary protocol |
| Fixed-size | Mỗi frame có kích thước cố định | Gói đơn giản |
| Self-describing | JSON/messagepack có parser | Dữ liệu cấu trúc |

Ví dụ dưới đây dùng newline-delimited để dễ quan sát.

### 43.2. TCP server async bằng Tokio

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("listening on 127.0.0.1:7000");

    loop {
        let (stream, peer) = listener.accept().await?;
        println!("accepted {peer}");
        tokio::spawn(async move {
            if let Err(error) = handle_client(stream).await {
                eprintln!("client {peer} failed: {error}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let reply = format!("echo: {line}\n");
        writer.write_all(reply.as_bytes()).await?;
    }

    Ok(())
}
```

`into_split` tách read half và write half, giúp đọc và ghi độc lập. Mỗi connection được đưa vào task riêng; cần giới hạn tài nguyên nếu server có thể nhận rất nhiều client.

### 43.3. TCP client async

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:7000").await?;
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    writer.write_all(b"hello\n").await?;
    writer.write_all(b"rust\n").await?;
    writer.shutdown().await?;

    while let Some(line) = lines.next_line().await? {
        println!("server: {line}");
    }
    Ok(())
}
```

`shutdown().await` đóng phía ghi, không nhất thiết đóng ngay toàn bộ stream; server có thể nhận EOF sau khi đọc hết dữ liệu. Nếu muốn giữ kết nối để gửi nhiều message, không gọi shutdown cho đến khi xong.

### 43.4. Timeout khi kết nối và đọc TCP

```rust
use std::time::Duration;
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

let stream = timeout(
    Duration::from_secs(3),
    TcpStream::connect("127.0.0.1:7000"),
).await??;

let mut buffer = [0_u8; 1024];
let read = timeout(Duration::from_secs(5), stream.read(&mut buffer)).await??;
println!("read {read} bytes");
```

Cần đặt timeout riêng cho connect, read idle, write và toàn bộ operation. TCP connection có thể còn tồn tại nhưng peer không gửi thêm byte; nếu không có read timeout, task có thể chờ vô hạn.

### 43.5. Dùng `TcpStream` đồng bộ

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7000")?;
    stream.write_all(b"hello\n")?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("{response}");
    Ok(())
}
```

Dùng `std::net` khi chương trình nhỏ, ít connection và không cần async. Không trộn blocking read/write vào Tokio worker thread.

### 43.6. TCP protocol với length prefix

Đối với binary message, một framing đơn giản là 4 byte big-endian chứa độ dài rồi đến payload:

```rust
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> std::io::Result<()> {
    let length = u32::try_from(payload.len())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "frame too large"))?;
    writer.write_u32(length).await?;
    writer.write_all(payload).await
}

async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Vec<u8>> {
    let length = reader.read_u32().await?;
    if length > 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame exceeds 1 MiB",
        ));
    }

    let mut payload = vec![0_u8; length as usize];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}
```

Luôn giới hạn frame size. Nếu đọc độ dài do client kiểm soát mà cấp phát vector không giới hạn, server có thể bị memory exhaustion.

### 43.7. TCP server có shared state

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct ServerState {
    connections: usize,
}

type SharedState = Arc<Mutex<ServerState>>;

async fn register_connection(state: &SharedState) {
    let mut value = state.lock().await;
    value.connections += 1;
}
```

Không giữ lock trong lúc network I/O. Lấy snapshot cần thiết, thả lock, rồi mới `await` đọc/ghi socket.

---

## 44. UDP socket: datagram, timeout và giới hạn tin cậy

### 44.1. UDP server

```rust
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:9000").await?;
    let mut buffer = [0_u8; 2048];

    loop {
        let (size, peer) = socket.recv_from(&mut buffer).await?;
        println!("received {size} bytes from {peer}: {:?}", &buffer[..size]);
        socket.send_to(&buffer[..size], peer).await?;
    }
}
```

### 44.2. UDP client

```rust
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(b"ping", "127.0.0.1:9000").await?;

    let mut buffer = [0_u8; 2048];
    let (size, peer) = timeout(
        Duration::from_secs(2),
        socket.recv_from(&mut buffer),
    ).await??;

    println!("reply from {peer}: {:?}", &buffer[..size]);
    Ok(())
}
```

UDP không đảm bảo giao hàng, thứ tự, không trùng lặp hoặc congestion control giống TCP. Nếu ứng dụng cần reliability, phải tự thêm sequence number, acknowledgement, retry, duplicate detection và timeout; hoặc dùng giao thức đã giải quyết các vấn đề đó.

### 44.3. `connect` cho UDP

```rust
let socket = UdpSocket::bind("0.0.0.0:0").await?;
socket.connect("127.0.0.1:9000").await?;
socket.send(b"ping").await?;
let mut buffer = [0_u8; 1024];
let size = socket.recv(&mut buffer).await?;
```

UDP `connect` không tạo handshake tin cậy như TCP. Nó gắn peer mặc định cho socket, cho phép dùng `send`/`recv` và lọc một số packet từ địa chỉ khác.

### 44.4. Broadcast và multicast

Broadcast/multicast phụ thuộc OS và network. Cần cấu hình socket options, địa chỉ multicast, interface và firewall. Không nên giả định `127.0.0.1` phản ánh behavior trên LAN. Hãy kiểm thử trong đúng môi trường triển khai và đặt TTL, giới hạn packet size, xác thực payload.

---

## 45. WebSocket với Tokio Tungstenite

WebSocket cung cấp kết nối hai chiều lâu dài với message/frame, thường bắt đầu bằng HTTP upgrade. `tokio-tungstenite` tích hợp Tungstenite với Tokio; WebSocket stream triển khai `Stream` và `Sink`, vì vậy có thể dùng `next()` để nhận và `send()` để gửi. [20]

### 45.1. WebSocket client echo

```rust
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut socket, response) = connect_async("ws://127.0.0.1:9100").await?;
    println!("handshake status: {}", response.status());

    socket.send(Message::Text("hello".into())).await?;

    while let Some(message) = socket.next().await {
        match message? {
            Message::Text(text) => {
                println!("text: {text}");
                break;
            }
            Message::Binary(bytes) => println!("binary: {} bytes", bytes.len()),
            Message::Ping(payload) => println!("ping: {} bytes", payload.len()),
            Message::Pong(_) => {}
            Message::Close(frame) => {
                println!("closed: {frame:?}");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

`Message::Text` và `Message::Binary` phụ thuộc API Tungstenite version; khi nâng version, đọc compiler hint vì kiểu payload có thể thay đổi. Đừng tự giả định mọi message là text.

### 45.2. WebSocket server trên TCP listener

```rust
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9100").await?;
    println!("websocket listening on 127.0.0.1:9100");

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = handle_ws(stream).await {
                eprintln!("{peer} failed: {error}");
            }
        });
    }
}

async fn handle_ws(stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut socket = accept_async(stream).await?;

    while let Some(message) = socket.next().await {
        let message = message?;
        if message.is_close() {
            break;
        }
        socket.send(message).await?;
    }

    Ok(())
}
```

### 45.3. Tách read/write task

```rust
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

async fn split_connection(
    socket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) {
    let (mut writer, mut reader) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    let write_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if writer.send(message).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(message)) = reader.next().await {
        println!("received: {message:?}");
        let _ = tx.send(Message::Text("ack".into())).await;
    }

    write_task.abort();
}
```

Channel có capacity tạo backpressure. Nếu producer nhanh hơn socket, `send().await` sẽ chờ thay vì tăng memory vô hạn.

### 45.4. `ws://`, `wss://` và TLS feature

`ws://` là WebSocket trên TCP; `wss://` là WebSocket trên TLS. Với `tokio-tungstenite`, chọn feature TLS tương ứng, ví dụ:

```toml
tokio-tungstenite = {
    version = "0.30",
    features = ["connect", "rustls-tls-native-roots"]
}
```

Tên feature cần kiểm tra theo version đang dùng bằng `cargo info tokio-tungstenite`. Không tắt xác minh certificate chỉ để làm cho demo chạy được. Trong production, hostname của URL phải khớp certificate và hệ thống phải có root certificate phù hợp.

---

## 46. MQTT với rumqttc: broker, topic, QoS và EventLoop

MQTT không kết nối client trực tiếp với client. Client kết nối broker, publish message lên topic hoặc subscribe topic. `rumqttc` có API đồng bộ và async; API async dùng `AsyncClient` cùng `EventLoop`. EventLoop phải được poll liên tục để kết nối tiến triển, nhận packet, gửi packet keep-alive và xử lý reconnect. Không được chặn trong vòng lặp `poll`. [19]

### 46.1. Mô hình MQTT

| Thành phần | Ý nghĩa |
|---|---|
| Broker | Server trung tâm nhận, lọc và chuyển message |
| Client ID | Định danh một MQTT client |
| Topic | Chuỗi phân cấp, ví dụ `devices/device-01/temperature` |
| Publish | Gửi payload lên topic |
| Subscribe | Đăng ký nhận message từ topic/filter |
| QoS 0 | At most once; có thể mất |
| QoS 1 | At least once; có thể nhận trùng, cần xử lý idempotent |
| QoS 2 | Exactly once ở mức giao thức, chi phí cao hơn |
| Retain | Broker giữ message cuối cho subscriber mới |
| Keep-alive | Cơ chế duy trì và phát hiện kết nối chết |
| Will | Message broker gửi nếu client mất kết nối bất thường |

### 46.2. Async publish/subscribe

```rust
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = MqttOptions::new("rust-demo-client", "127.0.0.1", 1883);
    options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(options, 32);
    client
        .subscribe("demo/temperature", QoS::AtLeastOnce)
        .await?;

    client
        .publish(
            "demo/temperature",
            QoS::AtLeastOnce,
            false,
            br#"{"value":25.5,"unit":"C"}"#,
        )
        .await?;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(message))) => {
                println!(
                    "topic={}, payload={:?}",
                    message.topic,
                    message.payload
                );
            }
            Ok(event) => {
                println!("mqtt event: {event:?}");
            }
            Err(error) => {
                eprintln!("mqtt event loop error: {error}");
                break;
            }
        }
    }

    Ok(())
}
```

Để chạy ví dụ, cần có MQTT broker ở `127.0.0.1:1883`, chẳng hạn Mosquitto hoặc broker tương thích. Code trên không tự cài broker và không nên coi broker công khai là môi trường test ổn định.

### 46.3. Tách publisher và event loop

`AsyncClient` gửi request vào event loop; event loop phải luôn được chạy. Một mẫu thường dùng là spawn task poll event loop rồi dùng client ở task chính:

```rust
use rumqttc::{AsyncClient, Event, MqttOptions};
use std::time::Duration;
use tokio::time::{sleep, Duration as TokioDuration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = MqttOptions::new("publisher-client", "127.0.0.1", 1883);
    options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(options, 64);

    let event_task = tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(event) => println!("event: {event:?}"),
                Err(error) => {
                    eprintln!("event loop stopped: {error}");
                    break;
                }
            }
        }
    });

    for value in 0..5 {
        client
            .publish(
                "demo/counter",
                rumqttc::QoS::AtLeastOnce,
                false,
                value.to_string(),
            )
            .await?;
        sleep(TokioDuration::from_millis(500)).await;
    }

    event_task.abort();
    Ok(())
}
```

Trong service thật, không nên abort ngay khi publish xong nếu cần chờ PUBACK hoặc flush. Hãy dùng shutdown channel và chờ event loop kết thúc có kiểm soát.

### 46.4. MQTT message là bytes: deserialize JSON

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Temperature {
    value: f32,
    unit: String,
}

fn parse_temperature(payload: &[u8]) -> Result<Temperature, serde_json::Error> {
    serde_json::from_slice(payload)
}
```

Trong handler:

```rust
match parse_temperature(&message.payload) {
    Ok(value) => println!("{} {}", value.value, value.unit),
    Err(error) => eprintln!("invalid MQTT payload: {error}"),
}
```

Không giả định payload luôn là UTF-8 hoặc JSON. Kiểm tra encoding, schema version, kích thước tối đa và nguồn topic trước khi parse.

### 46.5. Topic design

Nên thiết kế topic có namespace rõ ràng:

```text
tenant/{tenant_id}/device/{device_id}/telemetry/{metric}
tenant/{tenant_id}/device/{device_id}/command/{command}
tenant/{tenant_id}/device/{device_id}/state
```

Không đưa secret vào topic. Dùng ACL của broker để giới hạn client chỉ publish/subscribe đúng tenant và device. Phân biệt topic command và telemetry để policy QoS, retain và quyền truy cập khác nhau.

### 46.6. QoS và idempotency

QoS 1 có thể giao message nhiều lần. Consumer phải xử lý lặp an toàn:

```rust
#[derive(Debug, serde::Deserialize)]
struct Command {
    message_id: String,
    action: String,
}
```

Lưu `message_id` đã xử lý hoặc thiết kế command idempotent. Không tăng counter hai lần chỉ vì broker redeliver cùng message. Với message quan trọng, ghi nhận trạng thái xử lý và ACK nghiệp vụ riêng, không nhầm MQTT PUBACK với việc business operation đã hoàn tất.

### 46.7. Reconnect và backoff

rumqttc event loop hỗ trợ tiến trình reconnect khi tiếp tục poll. Ứng dụng vẫn cần xử lý lỗi, log có giới hạn, tránh spin loop và dùng shutdown. Với logic nghiệp vụ bên ngoài, có thể bọc vòng event loop bằng backoff:

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn reconnect_backoff(attempt: u32) {
    let seconds = 2_u64.saturating_pow(attempt.min(5));
    sleep(Duration::from_secs(seconds.min(30))).await;
}
```

Không tạo hàng chục client mới mỗi khi poll lỗi mà không giải phóng client cũ. Một service nên có một owner rõ ràng cho client/event loop và một cơ chế shutdown.

### 46.8. MQTT TLS

MQTT thường dùng port 8883 cho TLS, nhưng port và policy phụ thuộc broker. `rumqttc` có các feature TLS dựa trên `tokio-rustls` hoặc `tokio-native-tls`; hãy kiểm tra feature version đang dùng bằng `cargo info rumqttc`. TLS phải xác minh certificate và hostname; không dùng `set_dangerous` hoặc custom verifier bỏ qua kiểm tra trong production. Với broker dùng chứng chỉ tự ký, cài root CA nội bộ đúng cách thay vì tắt verification.

---

## 47. HTTP, MQTT, socket và WebSocket: cách tổ chức service chung

### 47.1. Tách transport khỏi domain logic

Không nên để logic nghiệp vụ nằm trực tiếp trong handler HTTP, MQTT callback và TCP loop cùng lúc. Tạo service domain:

```rust
pub struct DeviceService {
    // repository, config, metrics...
}

impl DeviceService {
    pub async fn record_temperature(
        &self,
        device_id: &str,
        value: f32,
    ) -> Result<(), ServiceError> {
        println!("{device_id}: {value}");
        Ok(())
    }
}
```

HTTP handler chỉ parse request rồi gọi `DeviceService`. MQTT consumer chỉ deserialize payload rồi gọi cùng service. WebSocket chỉ chuyển event thành message. Cách này giúp test domain mà không cần mở network port.

### 47.2. Trait abstraction cho upstream

```rust
use async_trait::async_trait;

#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn save(&self, device_id: &str, value: f32) -> Result<(), RepositoryError>;
}
```

Dependency:

```toml
async-trait = "0.1"
```

Hoặc với Rust hiện đại có thể dùng native async trait tùy MSRV và cách dùng trait object. Khi public library cần hỗ trợ nhiều compiler version, kiểm tra `rust-version` trước khi chọn syntax.

### 47.3. Channel giữa các transport

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
enum InboundEvent {
    Temperature { device_id: String, value: f32 },
    Command { device_id: String, action: String },
}

let (tx, mut rx) = mpsc::channel::<InboundEvent>(256);

let worker = tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        println!("process: {event:?}");
    }
});
```

MQTT task, HTTP handler hoặc WebSocket task gửi `InboundEvent`; worker xử lý tuần tự hoặc phân phối tiếp. Capacity phải được chọn có chủ đích. Khi channel đầy, producer có thể chờ, trả lỗi hoặc dùng policy drop tùy yêu cầu.

### 47.4. Broadcast cho nhiều WebSocket client

```rust
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel::<String>(128);
let mut subscriber = tx.subscribe();

tx.send("temperature updated".to_owned())?;

if let Ok(message) = subscriber.recv().await {
    println!("broadcast: {message}");
}
```

Broadcast phù hợp thông báo realtime không cần bảo đảm mọi client nhận mọi message. Nếu client chậm hơn buffer, receiver có thể nhận `Lagged`; application phải quyết định bỏ qua, snapshot lại state hoặc đóng client.

### 47.5. Semaphore giới hạn concurrency

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

let limit = Arc::new(Semaphore::new(16));
let permit = Arc::clone(&limit).acquire_owned().await?;

let task = tokio::spawn(async move {
    let _permit = permit;
    // downstream request; permit được trả khi task kết thúc
});
```

Dùng semaphore cho số request upstream đồng thời, số connection, số file hoặc số task tốn tài nguyên. Giới hạn concurrency không thay thế timeout; cần cả hai.

---

## 48. Cấu hình, secret và môi trường chạy

### 48.1. Configuration typed

```rust
use std::{env, net::SocketAddr, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub http_addr: SocketAddr,
    pub request_timeout: Duration,
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let http_addr = env::var("HTTP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
            .parse()?;
        let request_timeout = Duration::from_secs(
            env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "10".to_owned())
                .parse()?,
        );
        let mqtt_host = env::var("MQTT_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned());
        let mqtt_port = env::var("MQTT_PORT")
            .unwrap_or_else(|_| "1883".to_owned())
            .parse()?;

        Ok(Self { http_addr, request_timeout, mqtt_host, mqtt_port })
    }
}
```

Validate config ngay khi khởi động, không đợi request đầu tiên mới phát hiện port hoặc URL sai. Với password/token, không đưa field secret vào `Debug` hoặc log trực tiếp.

### 48.2. `.env` chỉ dành cho development

Có thể dùng `dotenvy` để nạp `.env` local:

```toml
dotenvy = "0.15"
```

```rust
fn main() {
    let _ = dotenvy::dotenv();
}
```

Không commit `.env` chứa secret. Production nên dùng secret manager, environment injection của deployment platform hoặc file permission phù hợp.

### 48.3. URL và certificate

Parse URL lúc khởi động:

```rust
let base_url = reqwest::Url::parse(&std::env::var("API_BASE_URL")?)?;
```

Không nối URL bằng string thủ công nếu có path/query do user cung cấp. Dùng URL builder để tránh lỗi encoding và SSRF. Nếu server cho phép user nhập URL để fetch, phải có allowlist hostname, chặn private IP/loopback/link-local và giới hạn redirect.

---

## 49. Xử lý lỗi mạng theo tầng

### 49.1. Phân loại lỗi

```rust
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("request timed out")]
    Timeout,
    #[error("transport error: {0}")]
    Transport(String),
    #[error("remote returned HTTP status {status}")]
    Http { status: u16 },
    #[error("invalid response payload: {0}")]
    Decode(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}
```

Đừng gom mọi lỗi vào `String` quá sớm. Giữ error source bằng `#[from]` khi phù hợp để log chain, nhưng chuyển ra API response bằng thông điệp an toàn.

### 49.2. Quyết định retry

| Lỗi | Retry thường phù hợp? | Điều kiện |
|---|---:|---|
| DNS tạm thời | Có thể | Có backoff, giới hạn attempts |
| Connect timeout | Có thể | Downstream có thể phục hồi |
| Read timeout | Có thể | Operation idempotent |
| HTTP 429 | Có | Tôn trọng `Retry-After` |
| HTTP 500/502/503/504 | Có thể | Exponential backoff, circuit breaker |
| HTTP 400 | Không | Sửa request |
| HTTP 401/403 | Không tự động | Refresh credential hoặc báo lỗi |
| JSON decode | Không | Contract/server bug cần chẩn đoán |
| MQTT QoS 1 redelivery | Không coi là lỗi ngay | Dedupe bằng message ID |
| TCP EOF | Chỉ reconnect | Xác định protocol/session state |

Retry có thể làm sự cố nặng hơn nếu mọi instance cùng retry ở một thời điểm. Dùng exponential backoff, jitter và giới hạn tổng thời gian.

### 49.3. Circuit breaker khái niệm

Khi downstream liên tục thất bại, circuit breaker mở và từ chối nhanh một thời gian, sau đó thử một số request half-open. Điều này bảo vệ thread, connection pool và downstream. Không nên tự viết circuit breaker không có test về race condition; có thể dùng crate chuyên dụng sau khi đánh giá license, maintenance và semantics.

### 49.4. Log structured

```rust
use tracing::{error, info, instrument};

#[instrument(skip(client), fields(endpoint = %url))]
async fn fetch(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    info!("sending request");
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        error!(%status, "upstream returned error");
    }
    Ok(body)
}
```

Khởi tạo subscriber:

```rust
tracing_subscriber::fmt()
    .with_env_filter("info")
    .init();
```

Không log access token, cookie, password, MQTT credential hoặc payload có dữ liệu cá nhân. Dùng request ID/correlation ID để nối log giữa HTTP, MQTT và worker.

---

## 50. TLS, HTTPS, MQTTS và WSS

### 50.1. TLS là lớp bảo mật, không phải protocol application

Rustls cung cấp TLS 1.2/1.3 và xử lý xác thực certificate; nó không tự mở TCP socket, không tự DNS và không tự đọc/ghi file. Khi đã dùng Tokio, `tokio-rustls` thường là integration phù hợp. [24]

Trong hầu hết ứng dụng:

| Nhu cầu | Lựa chọn đơn giản |
|---|---|
| HTTPS client | Reqwest với feature `rustls` |
| HTTPS server | Axum phía trên Hyper/Tokio với TLS integration |
| WSS client | `tokio-tungstenite` với feature rustls |
| MQTTS client | `rumqttc` với feature rustls |
| TLS raw custom protocol | `tokio-rustls` + `TcpStream` |

### 50.2. HTTPS client an toàn

```toml
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }
```

```rust
let client = reqwest::Client::builder()
    .https_only(true)
    .build()?;
let response = client.get("https://api.example.com/health").send().await?;
```

`https_only(true)` là một lớp bảo vệ để client không vô tình gọi HTTP plaintext. Không dùng `danger_accept_invalid_certs(true)` trong production. Nếu có CA nội bộ, nạp certificate cụ thể bằng API certificate thay vì vô hiệu hóa toàn bộ verification.

### 50.3. Certificate hostname

Certificate được cấp cho hostname, không phải chỉ cho địa chỉ IP tùy ý. Khi TLS handshake, client cần biết server name để kiểm tra certificate. Vì vậy, dùng `https://service.example.com` thường đúng hơn `https://192.0.2.10` nếu certificate không chứa IP đó. FAQ của rumqttc cũng cảnh báo kết nối TLS tới bare IP với certificate tự ký có thể thất bại. [19]

### 50.4. Raw TLS với Tokio Rustls: ý tưởng

Luồng khái quát:

```text
TcpStream::connect
        |
        v
TlsConnector.connect(server_name, tcp_stream)
        |
        v
TlsStream<TcpStream>
        |
        +--> AsyncRead / AsyncWrite
```

Code production cần chọn root store, crypto provider, server name, protocol version, certificate client nếu mTLS và xử lý shutdown. Không tự viết certificate verifier để bỏ qua lỗi chỉ vì demo gặp certificate self-signed.

### 50.5. mTLS

mTLS yêu cầu client cũng gửi certificate/private key. Quy trình gồm:

1. Server có CA tin cậy và kiểm tra client certificate.
2. Client có certificate chain và private key tương ứng.
3. Hai bên cấu hình root store và policy hostname/identity.
4. Private key phải được bảo vệ, không embed vào Git hoặc binary public.
5. Certificate rotation cần được thiết kế trước khi certificate hết hạn.

---

## 51. API server hoàn chỉnh: module, state, error và upstream client

Ví dụ này kết hợp các kiến thức trước đó nhưng giữ domain nhỏ để dễ mở rộng.

### 51.1. Cấu trúc

```text
src/
├── main.rs
├── lib.rs
├── config.rs
├── error.rs
├── state.rs
└── api/
    ├── mod.rs
    ├── health.rs
    └── users.rs
```

### 51.2. Library root

```rust
// src/lib.rs
pub mod api;
pub mod config;
pub mod error;
pub mod state;
```

### 51.3. State

```rust
// src/state.rs
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
}

pub type SharedState = Arc<AppState>;
```

### 51.4. API router

```rust
// src/api/mod.rs
mod health;
mod users;

use axum::{routing::get, Router};
use crate::state::SharedState;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health::get))
        .route("/users/{id}", get(users::get))
        .with_state(state)
}
```

### 51.5. Handler

```rust
// src/api/health.rs
pub async fn get() -> &'static str {
    "ok"
}
```

```rust
// src/api/users.rs
use axum::{extract::{Path, State}, response::Json};
use serde::Serialize;
use crate::{error::ApiError, state::SharedState};

#[derive(Serialize)]
pub struct UserView {
    pub id: u64,
    pub name: String,
}

pub async fn get(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<UserView>, ApiError> {
    let _ = state.http.get("https://example.com").build()?;
    Ok(Json(UserView { id, name: "An".to_owned() }))
}
```

### 51.6. Main

```rust
use std::sync::Arc;
use rust_network_demo::{api, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let http = reqwest::Client::builder()
        .https_only(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let state = Arc::new(AppState { http });
    let app = api::router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Ví dụ này cho thấy binary có thể gọi library crate bằng tên package/crate, còn các module trong library dùng `crate::...`. Nếu `main.rs` và `lib.rs` nằm cùng package nhưng là hai crate khác nhau, không dùng `crate::api` trong `main.rs` để trỏ vào library; phải dùng tên library crate.

---

## 52. Kiểm thử code mạng mà không phụ thuộc mạng thật

### 52.1. Test pure function trước

```rust
fn parse_temperature(payload: &[u8]) -> Result<f32, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct Payload { value: f32 }
    Ok(serde_json::from_slice::<Payload>(payload)?.value)
}

#[test]
fn parses_temperature() {
    let value = parse_temperature(br#"{"value": 21.5}"#).unwrap();
    assert_eq!(value, 21.5);
}
```

Parser, topic routing, retry delay, config validation và error mapping nên được test mà không mở socket.

### 52.2. Test HTTP server bằng router

Axum router có thể được test bằng request nội bộ với service trait, nhưng API cụ thể thay đổi theo version. Một mẫu thường dùng:

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
```

```rust
#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_works() {
        // let response = super::router().oneshot(
        //     Request::builder().uri("/health").body(Body::empty()).unwrap()
        // ).await.unwrap();
        // assert_eq!(response.status(), StatusCode::OK);
        let _ = (Body::empty(), Request::new(Body::empty()), StatusCode::OK);
    }
}
```

Khi triển khai thật, bỏ phần placeholder và tạo router không cần bind port. Test handler qua `oneshot` nhanh hơn test bằng process server thật.

### 52.3. TCP test bằng port ephemeral

```rust
use tokio::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:0").await?;
let address = listener.local_addr()?;
println!("test server at {address}");
```

Port `0` yêu cầu OS cấp port trống, tránh hard-code và giảm xung đột khi test chạy song song. Tạo server task, chạy client, rồi abort/await task và đóng listener.

### 52.4. Test timeout và lỗi

Không chỉ test happy path. Cần test:

| Nhóm | Case |
|---|---|
| Connect | DNS sai, port đóng, timeout |
| Read | Peer im lặng, EOF sớm, frame quá lớn |
| Write | Connection reset, backpressure |
| HTTP | 2xx, 4xx, 429, 5xx, malformed JSON |
| MQTT | broker mất, duplicate QoS 1, payload sai |
| WebSocket | close frame, ping/pong, invalid message |
| TLS | hostname sai, CA thiếu, certificate hết hạn |
| Shutdown | Ctrl+C trong lúc request đang chờ |

---

## 53. Chống lỗi và bảo mật khi mở cổng mạng

### 53.1. Không bind `0.0.0.0` tùy tiện

`127.0.0.1` chỉ mở local. `0.0.0.0` mở trên mọi interface IPv4 và có thể công khai dịch vụ ra LAN/Internet tùy firewall. Trong development dùng loopback; production chỉ bind public khi đã có reverse proxy, authentication, TLS, rate limit và firewall phù hợp.

### 53.2. Giới hạn input

Mọi protocol server cần giới hạn:

```text
- HTTP body size
- Header size
- JSON nesting/array size nếu phù hợp
- TCP frame length
- UDP datagram size
- MQTT payload/topic length theo policy
- WebSocket message size
- Số connection và số task
```

Đừng deserialize input không giới hạn vào cấu trúc có thể cấp phát lớn. Đừng gọi `.collect::<Vec<_>>()` trên stream vô hạn.

### 53.3. SSRF khi server gọi URL do user cung cấp

Nếu API nhận URL rồi server fetch, đây là rủi ro SSRF. Cần parse URL, allowlist scheme `https`, allowlist hostname, resolve DNS và kiểm tra địa chỉ sau resolve, chặn loopback/private/link-local/metadata endpoint, giới hạn redirect và response size. Không chỉ kiểm tra chuỗi URL bắt đầu bằng `https://`.

### 53.4. Secret và log

Không commit:

```text
MQTT_PASSWORD=...
API_TOKEN=...
TLS_PRIVATE_KEY=...
DATABASE_URL chứa password
```

Thêm `.env` vào `.gitignore`, rotate secret nếu lỡ commit, và dùng secret manager trong production. Redact header `Authorization`, cookie và payload nhạy cảm.

### 53.5. Dependency audit

Các lệnh hữu ích:

```bash
cargo tree
cargo tree -e features
cargo audit
cargo deny check
cargo outdated
```

`cargo audit`, `cargo deny` và `cargo outdated` cần cài riêng nếu môi trường chưa có. Đọc license, MSRV, maintenance, transitive dependency và native build requirement trước khi đưa crate vào service.

---

## 54. Xử lý lỗi compile thường gặp khi thêm crate mạng

### 54.1. `the trait bound ... is not satisfied`

Thường do thiếu feature hoặc import trait:

```rust
use futures_util::{SinkExt, StreamExt};
```

Nếu gọi `.send()`/`.next()` trên WebSocket mà không import `SinkExt`/`StreamExt`, compiler không thấy method extension.

### 54.2. `cannot find macro tokio::main`

Manifest phải bật macro/runtime:

```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

hoặc:

```toml
tokio = { version = "1", features = ["full"] }
```

### 54.3. Reqwest `.json()` không tồn tại

Bật feature:

```toml
reqwest = { version = "0.13", features = ["json"] }
```

Nếu đã `default-features = false`, phải khai báo rõ mọi feature cần dùng như `json`, `query`, `rustls`, `blocking`.

### 54.4. `runtime flavor ... requires rt-multi-thread`

Nếu dùng:

```rust
#[tokio::main(flavor = "multi_thread")]
```

phải bật `rt-multi-thread`. Nếu chỉ cần single-thread:

```rust
#[tokio::main(flavor = "current_thread")]
```

### 54.5. `future cannot be sent between threads safely`

Một giá trị không `Send` đang được giữ qua `.await` trong task được spawn. Cách xử lý:

1. Xác định biến nào sống qua `.await`.
2. Không giữ `MutexGuard` không phù hợp qua `.await`.
3. Dùng loại dữ liệu `Send + Sync`.
4. Chuyển dữ liệu thành owned value trước khi spawn.
5. Nếu task bắt buộc local, dùng local task set thay vì spawn đa luồng.

### 54.6. `borrowed value does not live long enough` với `tokio::spawn`

Sai mẫu:

```rust
let text = String::from("hello");
tokio::spawn(async {
    println!("{text}");
});
```

Đúng:

```rust
let text = String::from("hello");
tokio::spawn(async move {
    println!("{text}");
});
```

`move` chuyển ownership vào task. Với dữ liệu dùng chung, dùng `Arc::clone`.

### 54.7. `Address already in use`

Có process khác đang giữ port hoặc test trước chưa đóng listener. Dùng port `0` trong test, kiểm tra process:

```bash
ss -ltnp | grep 3000
```

Không xử lý bằng cách tăng port ngẫu nhiên trong production mà không cập nhật service discovery/firewall.

---

## 55. Lệnh chạy thực hành theo từng giao thức

| Mục tiêu | Lệnh |
|---|---|
| Kiểm tra tất cả target | `cargo check --all-targets --all-features` |
| Chạy API server | `cargo run --bin api-server` |
| Gọi HTTP example | `cargo run --example http-client` |
| Chạy TCP server | `cargo run --example tcp-server` |
| Chạy TCP client | `cargo run --example tcp-client` |
| Chạy UDP demo | `cargo run --example udp-demo` |
| Chạy WebSocket server | `cargo run --example websocket-server` |
| Chạy WebSocket client | `cargo run --example websocket-client` |
| Chạy MQTT client local | `cargo run --example mqtt-client` |
| Test unit/integration/doc | `cargo test --all-targets --all-features` |
| In log Tokio/HTTP | `RUST_LOG=info cargo run` |
| Kiểm tra dependency | `cargo tree -e features` |
| Kiểm tra package manifest | `cargo metadata --no-deps` |

Trình tự local thường là: khởi động broker nếu test MQTT, khởi động TCP/WebSocket/API server, sau đó chạy client. Với HTTP có thể dùng `curl`:

```bash
curl -i http://127.0.0.1:3000/health
curl -i http://127.0.0.1:3000/users/42
curl -i -X POST http://127.0.0.1:3000/users \
  -H 'content-type: application/json' \
  -d '{"name":"An"}'
```

Không dùng payload shell chứa secret thật trong history. Khi test MQTT, dùng client khác như `mosquitto_pub`/`mosquitto_sub` nếu đã cài, hoặc một ứng dụng Rust thứ hai.

---

## 56. Checklist production cho service Rust kết nối mạng

| Nhóm | Câu hỏi cần trả lời |
|---|---|
| Protocol | Có hiểu rõ semantics HTTP/TCP/UDP/MQTT/WebSocket chưa? |
| Timeout | Connect, read, write và total deadline đã có chưa? |
| Retry | Operation có idempotent không? Có backoff/jitter không? |
| TLS | Certificate được verify? Hostname đúng? CA rotation có kế hoạch? |
| Auth | Token/password không hard-code? Có rotation/revocation? |
| Input | Body/frame/payload/connection có giới hạn? |
| Backpressure | Channel/socket/downstream chậm thì hành vi gì? |
| Concurrency | Có giới hạn task, connection, request không? |
| Shutdown | SIGTERM/Ctrl+C có graceful shutdown không? |
| Observability | Có structured log, metric, tracing, request ID không? |
| Testing | Có test timeout, disconnect, malformed input, duplicate không? |
| Dependency | Feature, license, MSRV, advisory và native build đã kiểm tra? |
| Deployment | Bind address, firewall, health check và readiness đã đúng? |

Công thức đáng nhớ cho code mạng:

> **Timeout mọi I/O → giới hạn mọi input → xác thực mọi peer → retry có điều kiện → log có redaction → shutdown có kiểm soát.**

---

## 57. Tóm tắt lựa chọn nhanh

Nếu chỉ cần gọi một API HTTPS, dùng `reqwest::Client`. Nếu cần nhiều request đồng thời, dùng async Reqwest dưới Tokio. Nếu cần tạo REST server, dùng Axum với router, extractor, state và error response. Nếu cần giao thức riêng có thứ tự, dùng TCP nhưng tự định nghĩa framing. Nếu cần datagram nhẹ và chấp nhận mất gói, dùng UDP. Nếu cần browser realtime, dùng WebSocket. Nếu có nhiều thiết bị và broker, dùng MQTT. Nếu cần bảo mật, bật TLS đúng cách và giữ certificate verification.

Một kiến trúc tốt thường có các lớp:

```text
Transport layer
  HTTP / MQTT / TCP / UDP / WebSocket
        |
Protocol adapter
  parse, validate, frame, map error
        |
Application service
  business rules, authorization, idempotency
        |
Domain / repository
  typed data and persistence
```

Khi module hóa, mỗi lớp nên nằm trong module riêng và chỉ public những contract cần thiết:

```rust
mod transport;
mod protocol;
mod application;
mod domain;

pub use application::DeviceService;
pub use domain::{DeviceId, Temperature};
```

Binary hoặc server root lắp ráp các module; client bên ngoài chỉ dùng facade public. Đây là cách kết hợp module system với network programming mà không để implementation detail lan khắp project.

---

---

## Tài liệu tham khảo

[1]: https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html "The Rust Programming Language — Packages and Crates"

[2]: https://doc.rust-lang.org/cargo/reference/manifest.html "The Cargo Book — The Manifest Format"

[3]: https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html "The Rust Programming Language — Paths for Referring to an Item"

[4]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html "The Cargo Book — Specifying Dependencies"

[5]: https://doc.rust-lang.org/reference/visibility-and-privacy.html "The Rust Reference — Visibility and Privacy"

[6]: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html "The Rust Programming Language — Bringing Paths into Scope with use"

[7]: https://doc.rust-lang.org/cargo/reference/workspaces.html "The Cargo Book — Workspaces"

[8]: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html "The Rust Programming Language — Defining Modules to Control Scope and Privacy"

[9]: https://doc.rust-lang.org/reference/paths.html "The Rust Reference — Paths"

[10]: https://doc.rust-lang.org/reference/names/namespaces.html "The Rust Reference — Namespaces"

[11]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html "The Cargo Book — Cargo Targets"

[12]: https://doc.rust-lang.org/cargo/reference/features.html "The Cargo Book — Features"

[13]: https://doc.rust-lang.org/cargo/reference/build-scripts.html "The Cargo Book — Build Scripts"

[14]: https://doc.rust-lang.org/reference/macros-by-example.html "The Rust Reference — Macros by Example"

[15]: https://doc.rust-lang.org/reference/procedural-macros.html "The Rust Reference — Procedural Macros"

[16]: https://doc.rust-lang.org/cargo/commands/cargo-test.html "The Cargo Book — cargo test"


[17]: https://docs.rs/reqwest/latest/reqwest/ "Reqwest API documentation"

[18]: https://docs.rs/tokio/latest/tokio/net/ "Tokio networking API documentation"

[19]: https://docs.rs/rumqttc/latest/rumqttc/ "rumqttc MQTT client documentation"

[20]: https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/ "tokio-tungstenite WebSocket documentation"

[21]: https://docs.rs/axum/latest/axum/ "Axum HTTP routing and request handling documentation"

[22]: https://tokio.rs/tokio/tutorial "Tokio official tutorial"

[23]: https://docs.rs/serde_json/latest/serde_json/ "Serde JSON documentation"

[24]: https://docs.rs/rustls/latest/rustls/ "Rustls TLS documentation"

[25]: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html "Tokio timeout documentation"
---

**Tác giả:** Manus AI  
**Ngôn ngữ:** Tiếng Việt  
**Chủ đề:** Rust module system, crates, packages, paths, visibility, Cargo dependencies, workspaces, HTTP/API, MQTT, TCP/UDP, WebSocket, TLS và Tokio
