use std::str::FromStr;
use cpal::DeviceId;
use serde::{Deserializer, Serializer};
use serde::de::Error;

#[derive(Debug, Clone)]
pub struct ConfigDeviceId(DeviceId);

impl ConfigDeviceId {
    pub fn new(id: DeviceId) -> Self {
        Self(id)
    }

    pub fn id(&self) -> &DeviceId {
        &self.0
    }
}

impl serde::Serialize for ConfigDeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ConfigDeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(DeviceId::from_str(&s).map_err(D::Error::custom)?))
    }
}