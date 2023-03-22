use httpstatus::StatusCode;
use serde::{Deserialize, Deserializer, Serializer};

pub fn status_code_serialize<S>(val: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(val.as_u16())
}

pub fn status_code_from_u16<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<StatusCode, D::Error> {
    let code: u16 = Deserialize::deserialize(deserializer)?;

    Ok(StatusCode::from(code))
}
