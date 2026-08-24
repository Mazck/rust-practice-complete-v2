use modules_lab::{
    create_user,
    domain::{Email, Role},
};

#[test]
fn public_facade_is_usable() {
    let email = Email::parse("user@example.com").unwrap();
    assert_eq!(email.as_str(), "user@example.com");
    let user = create_user("User", email.as_str()).unwrap();
    assert_eq!(user.role(), Role::User);
}
