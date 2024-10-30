use serde::{
    de::{Deserializer, Error},
    Deserialize,
};

pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::unknown_variant(s, &["true", "false"])),
    }
}

pub fn deserialize_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    let Some(s) = s else {
        return Ok(None);
    };

    match s {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Bool(b) => Ok(Some(b)),
        serde_json::Value::Number(_) => Err(Error::unknown_variant("Number", &["true", "false"])),
        serde_json::Value::String(s) => match s.as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(Error::unknown_variant(s.as_str(), &["true", "false"])),
        },
        serde_json::Value::Array(_) => Err(Error::unknown_variant("Array", &["true", "false"])),
        serde_json::Value::Object(_) => Err(Error::unknown_variant("Object", &["true", "false"])),
    }
}
