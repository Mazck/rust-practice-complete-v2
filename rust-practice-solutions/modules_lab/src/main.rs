use modules_lab::{create_user, domain::Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut user = create_user("An", "an@example.com")?;
    println!("{} {} {:?}", user.name(), user.email(), user.role());

    // Constructor và field nội bộ được che; chỉ API method public được dùng.
    if user.role() == Role::User {
        println!("regular user");
    }

    // Avoid an unnecessary mutable binding in the demonstration output.
    let _ = &mut user;
    Ok(())
}
