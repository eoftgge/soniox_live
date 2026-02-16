use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, DeviceId};
use serde::de::Error;
use serde::{Deserializer, Serializer};
use std::str::FromStr;

pub struct MappableAvailableDevices(cpal::Host, Vec<AvailableDevice>);

#[derive(Clone)]
pub struct AvailableDevice {
    inner: Device,
    name: String,
    id: SettingDeviceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingDeviceId(DeviceId);

impl AvailableDevice {
    pub fn new(device: Device) -> Option<Self> {
        let id = device.id().ok()?;
        let desc = device.description().ok()?;
        Some(Self {
            inner: device,
            id: SettingDeviceId(id),
            name: desc.name().into(),
        })
    }

    pub fn from_host(host: &cpal::Host) -> Option<Self> {
        let device = host.default_output_device()?;
        Self::new(device)
    }

    pub fn into_inner(self) -> Device {
        self.inner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &SettingDeviceId {
        &self.id
    }
}

impl MappableAvailableDevices {
    pub fn from_host(host: cpal::Host) -> Self {
        let devices = host
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|d| AvailableDevice::new(d))
            .collect();
        Self(host, devices)
    }

    pub fn from_default_host() -> Self {
        let host = cpal::default_host();
        Self::from_host(host)
    }

    pub fn get(&self, id: &SettingDeviceId) -> Option<&AvailableDevice> {
        self.1.iter().find(|d| d.id() == id)
    }

    pub fn to_output_device(&self, id: Option<&SettingDeviceId>) -> Option<AvailableDevice> {
        if let Some(target) = id {
            self.get(target).cloned()
        } else {
            AvailableDevice::from_host(&self.0)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &AvailableDevice> {
        self.1.iter()
    }
}

impl SettingDeviceId {
    pub fn new(id: DeviceId) -> Self {
        Self(id)
    }

    pub fn inner(&self) -> &DeviceId {
        &self.0
    }
}

impl serde::Serialize for SettingDeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.0.to_string();
        s.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SettingDeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let id = DeviceId::from_str(&s).map_err(D::Error::custom)?;
        Ok(Self(id))
    }
}
