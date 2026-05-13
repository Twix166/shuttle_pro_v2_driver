use crate::keys::KeyChord;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize)]
pub struct Profile {
    pub profile: ProfileInfo,
    #[serde(default)]
    pub device: DeviceConfig,
    #[serde(default)]
    pub buttons: BTreeMap<String, ButtonConfig>,
    pub jog: JogConfig,
    pub shuttle: ShuttleConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileInfo {
    pub name: String,
    pub application: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceConfig {
    #[serde(default = "default_vendor_id")]
    pub vendor_id: u16,
    #[serde(default = "default_product_id")]
    pub product_id: u16,
    #[serde(default = "default_device_name")]
    pub name: String,
    #[serde(default = "default_button_base")]
    pub button_base: u16,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ButtonConfig {
    #[serde(default)]
    pub press: Vec<String>,
    #[serde(default)]
    pub release: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JogConfig {
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShuttleConfig {
    pub positive: Vec<String>,
    pub negative: Vec<String>,
    #[serde(default)]
    pub neutral: Vec<String>,
    #[serde(default = "default_shuttle_rates")]
    pub rates_per_second: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct CompiledProfile {
    pub profile: ProfileInfo,
    pub device: DeviceConfig,
    pub buttons: BTreeMap<u8, CompiledButton>,
    pub jog: CompiledJog,
    pub shuttle: CompiledShuttle,
}

#[derive(Clone, Debug, Default)]
pub struct CompiledButton {
    pub press: Vec<KeyChord>,
    pub release: Vec<KeyChord>,
}

#[derive(Clone, Debug)]
pub struct CompiledJog {
    pub positive: Vec<KeyChord>,
    pub negative: Vec<KeyChord>,
}

#[derive(Clone, Debug)]
pub struct CompiledShuttle {
    pub positive: Vec<KeyChord>,
    pub negative: Vec<KeyChord>,
    pub neutral: Vec<KeyChord>,
    pub rates_per_second: Vec<u16>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            vendor_id: default_vendor_id(),
            product_id: default_product_id(),
            name: default_device_name(),
            button_base: default_button_base(),
        }
    }
}

impl Profile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn compile(self) -> Result<CompiledProfile, Box<dyn std::error::Error>> {
        let mut buttons = BTreeMap::new();

        for (raw_number, button) in self.buttons {
            let number: u8 = raw_number
                .parse()
                .map_err(|_| format!("button key '{}' is not a number", raw_number))?;

            if !(1..=13).contains(&number) {
                return Err(format!("button {} is outside supported range 1..13", number).into());
            }

            buttons.insert(
                number,
                CompiledButton {
                    press: compile_chords(&button.press)?,
                    release: compile_chords(&button.release)?,
                },
            );
        }

        if self.shuttle.rates_per_second.len() != 8 {
            return Err(
                "shuttle.rates_per_second must contain exactly 8 entries for levels 0..7".into(),
            );
        }

        Ok(CompiledProfile {
            profile: self.profile,
            device: self.device,
            buttons,
            jog: CompiledJog {
                positive: compile_chords(&self.jog.positive)?,
                negative: compile_chords(&self.jog.negative)?,
            },
            shuttle: CompiledShuttle {
                positive: compile_chords(&self.shuttle.positive)?,
                negative: compile_chords(&self.shuttle.negative)?,
                neutral: compile_chords(&self.shuttle.neutral)?,
                rates_per_second: self.shuttle.rates_per_second,
            },
        })
    }
}

impl CompiledProfile {
    pub fn all_chords(&self) -> Vec<KeyChord> {
        let mut chords = Vec::new();

        for button in self.buttons.values() {
            chords.extend(button.press.clone());
            chords.extend(button.release.clone());
        }

        chords.extend(self.jog.positive.clone());
        chords.extend(self.jog.negative.clone());
        chords.extend(self.shuttle.positive.clone());
        chords.extend(self.shuttle.negative.clone());
        chords.extend(self.shuttle.neutral.clone());

        chords
    }
}

fn compile_chords(raw: &[String]) -> Result<Vec<KeyChord>, Box<dyn std::error::Error>> {
    raw.iter()
        .map(|key| KeyChord::from_str(key).map_err(|err| err.into()))
        .collect()
}

fn default_vendor_id() -> u16 {
    0x0b33
}

fn default_product_id() -> u16 {
    0x0030
}

fn default_device_name() -> String {
    "Contour ShuttlePro v2".to_string()
}

fn default_button_base() -> u16 {
    704
}

fn default_shuttle_rates() -> Vec<u16> {
    vec![0, 2, 4, 6, 8, 10, 12, 14]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_button_range() {
        let profile = Profile {
            profile: ProfileInfo {
                name: "test".to_string(),
                application: None,
            },
            device: DeviceConfig::default(),
            buttons: BTreeMap::from([("14".to_string(), ButtonConfig::default())]),
            jog: JogConfig {
                positive: vec!["right".to_string()],
                negative: vec!["left".to_string()],
            },
            shuttle: ShuttleConfig {
                positive: vec!["l".to_string()],
                negative: vec!["j".to_string()],
                neutral: vec!["k".to_string()],
                rates_per_second: default_shuttle_rates(),
            },
        };

        assert!(profile.compile().is_err());
    }
}
