use askama::Template;

#[derive(Template)]
#[template(path = "validation/username.html")]
pub struct UsernameValidation {
    pub is_valid: bool,
}
