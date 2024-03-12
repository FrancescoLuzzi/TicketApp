use askama::Template;

#[derive(Template)]
#[template(path = "validation/form.html")]
pub struct FormValidation<'a> {
    pub target: &'a str,
    pub valid_message: &'a str,
    pub invalid_message: &'a str,
    pub is_valid: bool,
}
