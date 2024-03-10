use askama::Template;

#[derive(Template)]
#[template(path = "validation/password.html")]
pub struct PasswordValidation {}
