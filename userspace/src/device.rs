use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputDevice {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub event: PathBuf,
}

pub fn find(vendor_id: u16, product_id: u16, name: &str) -> io::Result<Option<InputDevice>> {
    find_in_devices(
        &fs::read_to_string("/proc/bus/input/devices")?,
        vendor_id,
        product_id,
        name,
    )
}

pub fn find_in_devices(
    devices: &str,
    vendor_id: u16,
    product_id: u16,
    name: &str,
) -> io::Result<Option<InputDevice>> {
    for block in devices.split("\n\n") {
        let parsed = parse_block(block);

        if let Some(device) = parsed {
            if device.vendor_id == vendor_id
                && device.product_id == product_id
                && device.name == name
            {
                return Ok(Some(device));
            }
        }
    }

    Ok(None)
}

fn parse_block(block: &str) -> Option<InputDevice> {
    let mut name = None;
    let mut vendor_id = None;
    let mut product_id = None;
    let mut event = None;

    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("N: Name=\"") {
            name = rest.strip_suffix('"').map(ToOwned::to_owned);
        } else if let Some(rest) = line.strip_prefix("I: ") {
            for item in rest.split_whitespace() {
                if let Some(value) = item.strip_prefix("Vendor=") {
                    vendor_id = u16::from_str_radix(value, 16).ok();
                } else if let Some(value) = item.strip_prefix("Product=") {
                    product_id = u16::from_str_radix(value, 16).ok();
                }
            }
        } else if let Some(rest) = line.strip_prefix("H: Handlers=") {
            event = rest
                .split_whitespace()
                .find(|handler| handler.starts_with("event"))
                .map(|handler| PathBuf::from("/dev/input").join(handler));
        }
    }

    Some(InputDevice {
        name: name?,
        vendor_id: vendor_id?,
        product_id: product_id?,
        event: event?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_shuttlepro_block() {
        let devices = r#"I: Bus=0003 Vendor=0b33 Product=0030 Version=0111
N: Name="Contour ShuttlePro v2"
H: Handlers=event26
"#;

        let device = find_in_devices(devices, 0x0b33, 0x0030, "Contour ShuttlePro v2")
            .unwrap()
            .unwrap();

        assert_eq!(device.event, PathBuf::from("/dev/input/event26"));
    }
}
