#[derive(thiserror::Error, Debug)]
pub enum CtxError {
    #[error("invalid user id")]
    InvalidUserId,
}
