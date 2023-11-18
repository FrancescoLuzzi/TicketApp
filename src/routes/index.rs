use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {}

const INDEX: Index = Index {};

pub async fn index() -> Index {
    INDEX
}
