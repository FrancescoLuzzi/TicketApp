pub mod error;
use error::CtxError;

use uuid::Uuid;

use crate::auth::session_key::SessionKey;
#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: Uuid,
    session_id: SessionKey,
}

// Constructors.
impl Ctx {
    pub fn root_ctx() -> Self {
        Ctx {
            user_id: Uuid::default(),
            session_id: SessionKey::default(),
        }
    }

    pub fn new(user_id: Uuid, session_id: SessionKey) -> Result<Self, CtxError> {
        if user_id == uuid::Uuid::default() {
            Err(CtxError::InvalidUserId)
        } else {
            Ok(Self {
                user_id,
                session_id,
            })
        }
    }
}

// Property Accessors.
impl Ctx {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn session_id(&self) -> SessionKey {
        self.session_id.clone()
    }
}
