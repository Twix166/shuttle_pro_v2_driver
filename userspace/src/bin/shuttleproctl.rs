use clap::{Parser, Subcommand};
use shuttlepro::config::Profile;
use shuttlepro::device;
use shuttlepro::input::{EventDevice, ABS_MISC, EV_ABS, EV_KEY, EV_REL, REL_DIAL};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Test and inspect Contour ShuttlePro v2 userspace profiles"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Detect {
        #[arg(long, default_value_t = 0x0b33)]
        vendor: u16,
        #[arg(long, default_value_t = 0x0030)]
        product: u16,
        #[arg(long, default_value = "Contour ShuttlePro v2")]
        name: String,
    },
    Monitor {
        #[arg(long)]
        event: Option<PathBuf>,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
}

#[derive(Subcommand)]
enum ProfileCommand {
    Validate { file: PathBuf },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Detect {
            vendor,
            product,
            name,
        } => {
            let Some(device) = device::find(vendor, product, &name)? else {
                return Err("ShuttlePro event device not found".into());
            };

            println!("{}", device.event.display());
        }
        Command::Monitor { event } => {
            let event = match event {
                Some(path) => path,
                None => {
                    device::find(0x0b33, 0x0030, "Contour ShuttlePro v2")?
                        .ok_or("ShuttlePro event device not found")?
                        .event
                }
            };
            let device = EventDevice::open(&event, false)?;

            loop {
                if let Some(input) = device.read_event()? {
                    print_event(input.event_type, input.code, input.value);
                }
            }
        }
        Command::Profile {
            command: ProfileCommand::Validate { file },
        } => {
            let profile = Profile::load(&file)?.compile()?;
            println!("valid profile: {}", profile.profile.name);
        }
    }

    Ok(())
}

fn print_event(event_type: u16, code: u16, value: i32) {
    match event_type {
        EV_KEY => println!("button code={} value={}", code, value),
        EV_REL if code == REL_DIAL => println!("jog delta={}", value),
        EV_ABS if code == ABS_MISC => println!("shuttle value={}", value),
        _ => {}
    }
}
