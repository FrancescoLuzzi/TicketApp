pub mod error;
use error::CtxError;

use uuid::Uuid;
#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: Uuid,
}

// Constructors.
impl Ctx {
    pub fn root_ctx() -> Self {
        Ctx {
            user_id: Uuid::default(),
        }
    }

    pub fn new(user_id: Uuid) -> Result<Self, CtxError> {
        if user_id == uuid::Uuid::default() {
            Err(CtxError::InvalidUserId)
        } else {
            Ok(Self { user_id })
        }
    }
}

// Property Accessors.
impl Ctx {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
}
