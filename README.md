# Contour ShuttlePro v2 Linux HID Driver

Small out-of-tree HID driver for the Contour ShuttlePro v2 USB controller
(`0b33:0030`).

<p align="center">
  <img
    src="https://cdn.prod.website-files.com/68bad4196ea4422858dc1574/69d58e48c18051787f48bb7d_ShuttleProV2FRONT_Originalratio_1920x2880_U_100_1x1_1920x1920_U_100.png"
    alt="Contour ShuttlePro v2 USB controller"
    width="420">
</p>

Product image source: [Contour Design Shuttle Pro V2 product page](https://www.contourdesign.com/product/contour-shuttle-pro-v2).

## Status and Risk

This is an experimental out-of-tree Linux kernel module. It has been tested on
one ShuttlePro v2 device and one Fedora kernel, but it is not an official
Contour Design driver and is not part of the upstream Linux kernel.

Use this software entirely at your own risk. Kernel modules run with full
kernel privileges and can crash, hang, or otherwise destabilize the system.
There is no warranty; see [LICENSE](LICENSE).

## Creation Note

The initial driver, documentation, DKMS metadata, and helper scripts in this
repository were created with OpenAI Codex using the GPT-5.5 model, with live
testing against an attached Contour ShuttlePro v2 device.

The driver exposes raw device controls through evdev:

- `BTN_TRIGGER_HAPPY1` through `BTN_TRIGGER_HAPPY13` for the 13 physical
  buttons currently described by the HID report descriptor.
- `REL_DIAL` for jog-wheel movement.
- `ABS_MISC` for the spring-loaded shuttle wheel, with values `-7..7`.

It intentionally does not map controls to keyboard shortcuts. Application
profiles and macros belong in userspace.

The companion userspace mapper is under [userspace/](userspace/). It provides
`shuttleproctl` for device/profile testing and `shuttleprod` for profile-driven
keyboard mapping through Linux `uinput`, including a bundled Kdenlive profile.

Userspace command summary:

- `shuttleproctl detect` prints the current ShuttlePro event node.
- `shuttleproctl monitor` prints raw decoded device events.
- `shuttleproctl tui` opens a live terminal dashboard for hardware UAT.
- `shuttleproctl profile validate <file>` validates TOML profiles.
- `shuttleprod --profile <file>` runs the profile mapper daemon.

## Build

```sh
make
```

## Temporary Load

Load the module:

```sh
sudo insmod hid-shuttlepro.ko
```

If the device was already plugged in and did not bind automatically, move it
from `hid-generic` to `hid-shuttlepro`:

```sh
dev=$(basename /sys/bus/hid/devices/0003:0B33:0030.*)
echo -n "$dev" | sudo tee /sys/bus/hid/drivers/hid-generic/unbind
echo -n "$dev" | sudo tee /sys/bus/hid/drivers/hid-shuttlepro/bind
```

If unbind reports `No such device` and bind reports `Device or resource busy`,
the module is already bound. Confirm with:

```sh
readlink /sys/bus/hid/devices/0003:0B33:0030.*/driver
```

## DKMS Install

DKMS rebuilds the module automatically when new matching kernel headers are
installed:

```sh
sudo dkms add .
sudo dkms build hid-shuttlepro/0.1.0
sudo dkms install hid-shuttlepro/0.1.0
sudo modprobe hid-shuttlepro
```

For an automated DKMS plus userspace install, use:

```sh
scripts/install.sh
```

See [docs/INSTALL.md](docs/INSTALL.md) for installer options, post-install
checks, and the packaging roadmap.

To install and immediately enable the bundled Kdenlive userspace profile:

```sh
scripts/install.sh --configure-kdenlive
```

Load the module automatically at boot:

```sh
sudo install -m 0644 hid-shuttlepro.modules-load.conf \
  /etc/modules-load.d/hid-shuttlepro.conf
```

Allow the active desktop user to read the evdev node:

```sh
sudo install -m 0644 99-hid-shuttlepro.rules \
  /etc/udev/rules.d/99-hid-shuttlepro.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=input
```

## Test

Automated checks:

```sh
tests/ci.sh
tests/dkms-isolated.sh
```

The automated suite covers:

- smoke build of the kernel module with `W=1`;
- source style via `checkpatch.pl` when available;
- shell syntax and `shellcheck` when available;
- parser regression tests for malformed reports, clamping, jog deltas,
  wraparound, and button masking;
- optional KUnit parser test module build when kernel headers enable KUnit;
- unit tests for helper-script event-node discovery;
- regression checks for license, DKMS metadata, udev rules, and risk notice;
- isolated DKMS add/build flow without installing the module system-wide.
- optional `sparse` and Coccinelle checks when those tools are available.

Hardware integration test:

```sh
scripts/find-event.sh
sudo scripts/test-events.sh
```

Optional local KUnit parser test:

```sh
scripts/run-kunit.sh
```

`scripts/find-event.sh` prints the current event node for `Contour ShuttlePro
v2`; the number may change across reloads. Verify:

- each button emits a stable `BTN_TRIGGER_HAPPY*` press and release;
- the spring wheel emits `ABS_MISC` values from `-7` to `7` and returns to `0`;
- the jog wheel emits positive and negative `REL_DIAL` deltas.

The jog wheel reports an internal 8-bit absolute counter. The first report is
used as the baseline, so no jog delta is emitted until a later report.

The GitHub Actions pipeline runs the automated checks on every push and pull
request. It cannot run the hardware integration test because CI runners do not
have a Contour ShuttlePro v2 attached.

On pushes to `main`, CI failures automatically open or update a GitHub issue so
hardening regressions are tracked until the pipeline passes again.

See [docs/HARDENING.md](docs/HARDENING.md) for the full security and stability
validation plan, including sanitizer and hardware stress-test lanes.
See [docs/UAT.md](docs/UAT.md) for a step-by-step hardware acceptance test,
including the `shuttleproctl` TUI and Kdenlive profile checks.

Userspace mapper checks are included in CI when Rust is available:

```sh
cd userspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin shuttleproctl -- profile validate profiles/kdenlive.toml
```

For a non-invasive userspace hardware check:

```sh
cd userspace
cargo run --bin shuttleproctl -- detect
cargo run --bin shuttleproctl -- monitor
cargo run --bin shuttleproctl -- tui
cargo run --bin shuttleprod -- --profile profiles/kdenlive.toml --dry-run
```

## License

This project is licensed under GPL-2.0-only, matching normal Linux kernel
driver licensing practice. Source files use SPDX identifiers where applicable.
