use anyhow::anyhow;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "MOVEMENT_DIRECTION", rename_all = "lowercase")]
pub enum TicketDirection {
    Out,
    In,
}

impl TryFrom<&str> for TicketDirection {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            err_val => Err(anyhow!(
                "can't convert value '{err_val}' to TicketDirection"
            )),
        }
    }
}

impl TryFrom<String> for TicketDirection {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}
