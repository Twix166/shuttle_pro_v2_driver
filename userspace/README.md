# ShuttlePro Userspace Tools

This directory contains a small userspace companion for the kernel driver.
It reads the ShuttlePro evdev device and maps controls to profile-defined
keyboard events through Linux `uinput`.

## Build

```sh
cd userspace
cargo build --release
```

The repository installer can build and install both userspace binaries:

```sh
../scripts/install.sh --no-dkms
```

To install the bundled Kdenlive profile and enable the mapper user service:

```sh
../scripts/install.sh --configure-kdenlive
```

## Test the Device

```sh
cargo run --bin shuttleproctl -- detect
cargo run --bin shuttleproctl -- monitor
cargo run --bin shuttleproctl -- tui
```

`monitor` prints button, jog, and shuttle events without creating a virtual
keyboard.

`tui` opens a live terminal dashboard with button state, shuttle position, jog
movement, and a recent event log. Press `q`, `Esc`, or `Ctrl+C` to exit.

For a full hardware acceptance checklist, see [../docs/UAT.md](../docs/UAT.md).

Expected controls from the kernel driver:

- 13 buttons as `EV_KEY` codes starting at `BTN_TRIGGER_HAPPY1`;
- jog wheel movement as `REL_DIAL`;
- spring-loaded shuttle ring as `ABS_MISC` values from `-7` to `7`.

## Validate Profiles

```sh
cargo run --bin shuttleproctl -- profile validate profiles/kdenlive.toml
```

## Run the Kdenlive Profile

```sh
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml
```

For a safe first run that prints mapped actions without sending virtual
keyboard events:

```sh
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml --dry-run
```

The Kdenlive profile emits documented default keyboard shortcuts:

- jog wheel: `Left` / `Right` frame stepping;
- shuttle ring: repeated `J` / `L`, with `K` at neutral;
- buttons: common playback, zone, edit, save, undo, and clipboard actions.

The daemon opens `/dev/uinput` and the ShuttlePro `/dev/input/event*` node.
Depending on local permissions, you may need the repository udev rule installed
and the `uinput` kernel module loaded.

```sh
sudo modprobe uinput
```

By default, the daemon exclusively grabs the ShuttlePro event node so raw device
events do not leak to other applications. For debugging with another reader,
pass `--no-grab`.

Stop the daemon with `Ctrl+C` or `SIGTERM`; it releases the event-device grab
while shutting down.

## First-Run Checklist

1. Build the kernel module and confirm `scripts/find-event.sh` finds the device.
2. Install the repository udev rule so the desktop user can read the input node.
3. Load `uinput` and confirm `/dev/uinput` exists.
4. Run `shuttleproctl monitor` and test all buttons, jog, and shuttle ring.
5. Run `shuttleproctl tui` and confirm the dashboard reflects the same controls.
6. Run `shuttleprod --dry-run` and confirm actions match the selected profile.
7. Start Kdenlive, run `shuttleprod`, and confirm jog, shuttle, and key buttons.
