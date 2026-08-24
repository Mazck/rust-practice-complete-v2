fn main() {
    println!("version={}", cargo_lab::APP_VERSION);
    println!("render={}", cargo_lab::render("Rust"));
    println!("macro={}", cargo_lab::format_public!("ok"));
}
