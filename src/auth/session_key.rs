use bb8_redis::redis::{self, ToRedisArgs};
use derive_more::{Display, From};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng as _};

/// https://github.com/actix/actix-extras/blob/master/actix-session/src/storage
/// A session key, the string stored in a client-side cookie to associate a user with its session
/// state on the backend.
///
/// # Validation
/// Session keys are stored as cookies, therefore they cannot be arbitrary long. Session keys are
/// required to be smaller than 4064 bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionKey(String);

impl TryFrom<&str> for SessionKey {
    type Error = InvalidSessionKeyError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        if val.len() > 4064 {
            return Err(anyhow::anyhow!(
                "The session key is bigger than 4064 bytes, the upper limit on cookie content."
            )
            .into());
        }

        Ok(SessionKey(val.to_owned()))
    }
}

impl TryFrom<String> for SessionKey {
    type Error = InvalidSessionKeyError;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        val.as_str().try_into()
    }
}

impl AsRef<str> for SessionKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SessionKey> for String {
    fn from(key: SessionKey) -> Self {
        key.0
    }
}

impl ToRedisArgs for SessionKey {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg_fmt("{self}");
    }
}

#[derive(Debug, Display, From)]
#[display(fmt = "The provided string is not a valid session key")]
pub struct InvalidSessionKeyError(anyhow::Error);

impl std::error::Error for InvalidSessionKeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

pub(crate) fn generate_session_key() -> SessionKey {
    let value = std::iter::repeat(())
        .map(|()| OsRng.sample(Alphanumeric))
        .take(64)
        .collect::<Vec<_>>();

    // These unwraps will never panic because pre-conditions are always verified
    // (i.e. length and character set)
    String::from_utf8(value).unwrap().try_into().unwrap()
}
