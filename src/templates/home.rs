use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Home {
    pub user: Option<String>,
}