use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Nested<T: serde::de::DeserializeOwned> {
    pub inner: T,
}

impl<'de, T: serde::de::DeserializeOwned> serde::de::Deserialize<'de> for Nested<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::String(str) = str else {
            return Err(serde::de::Error::unknown_variant("!str", &["str"]));
        };

        let val: T =
            serde_json::from_str(&str).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Self { inner: val })
    }
}

impl<T: serde::de::DeserializeOwned> Deref for Nested<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
