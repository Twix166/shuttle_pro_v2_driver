use clap::Parser;
use shuttlepro::config::Profile;
use shuttlepro::device;
use shuttlepro::input::EventDevice;
use shuttlepro::keys::KeyChord;
use shuttlepro::mapper::{Mapper, MapperAction};
use shuttlepro::uinput::VirtualKeyboard;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(version, about = "Profile mapper daemon for the Contour ShuttlePro v2")]
struct Cli {
    #[arg(long)]
    profile: PathBuf,
    #[arg(long)]
    event: Option<PathBuf>,
    #[arg(long, help = "Do not exclusively grab the ShuttlePro event device")]
    no_grab: bool,
    #[arg(long, help = "Print mapped actions instead of opening /dev/uinput")]
    dry_run: bool,
}

trait Output {
    fn tap_chord(&mut self, chord: &KeyChord) -> Result<(), Box<dyn Error>>;
}

struct UInputOutput {
    keyboard: VirtualKeyboard,
}

struct DryRunOutput;

impl Output for UInputOutput {
    fn tap_chord(&mut self, chord: &KeyChord) -> Result<(), Box<dyn Error>> {
        self.keyboard.tap_chord(chord)?;
        Ok(())
    }
}

impl Output for DryRunOutput {
    fn tap_chord(&mut self, chord: &KeyChord) -> Result<(), Box<dyn Error>> {
        println!("tap {}", chord);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    run().map_err(|err| {
        eprintln!("shuttleprod: {err}");
        err
    })
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let profile = Profile::load(&args.profile)?.compile()?;
    let event = match args.event {
        Some(path) => path,
        None => {
            device::find(
                profile.device.vendor_id,
                profile.device.product_id,
                &profile.device.name,
            )?
            .ok_or(
                "ShuttlePro event device not found; run `shuttleproctl detect` to confirm the kernel driver event node",
            )?
            .event
        }
    };

    let mut device = EventDevice::open(&event, true)
        .map_err(|err| format!("failed to open {}: {err}", event.display()))?;
    if !args.no_grab {
        device.grab(true).map_err(|err| {
            format!(
                "failed to grab {}; close other readers or pass --no-grab for debugging: {err}",
                event.display()
            )
        })?;
    }

    let mut output: Box<dyn Output> = if args.dry_run {
        Box::new(DryRunOutput)
    } else {
        Box::new(UInputOutput {
            keyboard: VirtualKeyboard::create(
                "ShuttlePro v2 profile keyboard",
                &profile.all_chords(),
            )
            .map_err(|err| {
                format!("failed to create virtual keyboard at /dev/uinput; load the uinput module and check permissions: {err}")
            })?,
        })
    };

    let mut mapper = Mapper::new(profile);
    let mut repeat: Option<(Vec<_>, Duration, Instant)> = None;
    let shutdown = Arc::new(AtomicBool::new(false));

    flag::register(SIGINT, Arc::clone(&shutdown))?;
    flag::register(SIGTERM, Arc::clone(&shutdown))?;

    while !shutdown.load(Ordering::Relaxed) {
        while let Some(event) = device.read_event()? {
            for action in mapper.handle_event(event) {
                match action {
                    MapperAction::Chords(chords) => {
                        for chord in chords {
                            output.tap_chord(&chord)?;
                        }
                    }
                    MapperAction::StartRepeat { chords, interval } => {
                        repeat = Some((chords, interval, Instant::now()));
                    }
                    MapperAction::StopRepeat => repeat = None,
                }
            }
        }

        if let Some((chords, interval, next_at)) = repeat.as_mut() {
            if Instant::now() >= *next_at {
                for chord in chords {
                    output.tap_chord(chord)?;
                }
                *next_at = Instant::now() + *interval;
            }
        }

        thread::sleep(Duration::from_millis(2));
    }

    Ok(())
}
