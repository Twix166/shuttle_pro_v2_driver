# ShuttlePro v2 User Acceptance Test

This checklist validates the kernel driver, `shuttleproctl`, the TUI, and the
Kdenlive userspace profile against a real Contour ShuttlePro v2.

Run commands from the repository root unless noted otherwise.

## Preconditions

- The ShuttlePro v2 is plugged in.
- The kernel module is loaded and bound to the device.
- The repository udev rule is installed if testing as a desktop user.
- Rust is installed for local userspace builds.
- `uinput` is loaded before testing real profile output:

```sh
sudo modprobe uinput
```

## Build and Static Checks

```sh
tests/ci.sh
cd userspace
cargo build
cargo test
cargo run --bin shuttleproctl -- profile validate profiles/kdenlive.toml
cargo run --bin shuttleproctl -- profile validate profiles/test.toml
```

Pass criteria:

- CI completes without errors.
- Both bundled profiles validate.
- Local toolchains without `rustfmt` or `clippy` may skip those checks; GitHub
  Actions installs and runs them.

## Device Discovery

```sh
cd userspace
cargo run --bin shuttleproctl -- detect
```

Pass criteria:

- The command prints one `/dev/input/event*` path.
- The path matches the driver event node reported by `../scripts/find-event.sh`.

## Raw Event Monitor

```sh
cargo run --bin shuttleproctl -- monitor
```

Operate every hardware control.

Pass criteria:

- Each of the 13 buttons emits one press and one release.
- The jog wheel emits positive and negative `jog delta` lines.
- The shuttle ring emits values from negative through positive and returns to
  `shuttle value=0` when released.

Exit with `Ctrl+C`.

## TUI Hardware Dashboard

```sh
cargo run --bin shuttleproctl -- tui
```

Operate every hardware control.

Pass criteria:

- The status panel shows the detected event device.
- Button cells highlight while the matching physical button is held.
- The shuttle gauge moves left and right and returns to the center at rest.
- The jog panel updates direction, last delta, and total movement.
- The recent event log records button, jog, and shuttle activity.
- `q`, `Esc`, and `Ctrl+C` exit and restore the terminal.

Optional low-refresh mode:

```sh
cargo run --bin shuttleproctl -- tui --fps 10
```

## Profile Dry Run

```sh
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml --dry-run --no-grab
```

Operate representative controls.

Pass criteria:

- Button 1 prints `tap space`.
- Jog clockwise/counter-clockwise prints `tap right` / `tap left`.
- Shuttle left/right prints `tap j` / `tap l`.
- Releasing the shuttle prints `tap k`.

Exit with `Ctrl+C`.

## Kdenlive Integration

Start Kdenlive, then run:

```sh
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml
```

Pass criteria:

- Jog wheel steps frames left and right.
- Shuttle ring drives reverse/forward playback through Kdenlive shortcut
  repeats.
- Releasing the shuttle pauses playback.
- Button mappings in `profiles/kdenlive.toml` trigger their documented actions.
- Stopping `shuttleprod` with `Ctrl+C` releases the event-device grab.

If another process needs to read the device while debugging, run:

```sh
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml --no-grab
```

## Failure Notes

- If `/dev/input/event*` cannot be opened, install the udev rule and reconnect
  the device.
- If `/dev/uinput` cannot be opened, load `uinput` and check permissions.
- If Kdenlive actions do not match, verify Kdenlive keyboard shortcuts have not
  been customized away from the profile defaults.
