# Installation

This project currently provides a shell installer for local systems. The target
packaging model is:

- DKMS package for the out-of-tree HID kernel module;
- distro package for `shuttleproctl`, `shuttleprod`, profiles, udev rules, and
  the optional systemd user service;
- release artifacts built by CI once the driver and userspace ABI settle.

## Script Installer

Run from the repository root as your normal desktop user:

```sh
scripts/install.sh
```

The installer uses `sudo` for system-level installation steps and keeps Cargo
build artifacts owned by your user.

It installs:

- DKMS module `hid-shuttlepro/0.1.0`;
- `/etc/udev/rules.d/99-hid-shuttlepro.rules`;
- `/etc/modules-load.d/hid-shuttlepro.conf`;
- `shuttleproctl` and `shuttleprod` under `/usr/local/bin`;
- bundled profiles under `~/.config/shuttlepro/profiles`;
- optional systemd user service under `~/.config/systemd/user`.

## Options

```sh
scripts/install.sh --help
scripts/install.sh --prefix "$HOME/.local"
scripts/install.sh --no-dkms
scripts/install.sh --no-userspace
scripts/install.sh --no-user-service
scripts/install.sh --configure-kdenlive
```

Use `--prefix "$HOME/.local"` if you want userspace binaries installed without
using `/usr/local/bin`. DKMS, udev, and module-load configuration still require
`sudo`.

Use `--configure-kdenlive` to validate the bundled Kdenlive profile and enable
the `shuttleprod` systemd user service immediately. This is the closest current
option to out-of-the-box Kdenlive behavior.

The Kdenlive integration deliberately does not rewrite Kdenlive's shortcut
configuration files. It emits Kdenlive's documented default keyboard shortcuts
through `uinput`, so users with customized Kdenlive shortcuts should adjust
`~/.config/shuttlepro/profiles/kdenlive.toml` instead.

## Post-Install Checks

```sh
shuttleproctl detect
shuttleproctl tui
shuttleprod --profile "$HOME/.config/shuttlepro/profiles/kdenlive.toml" --dry-run --no-grab
```

For full hardware acceptance testing, follow [UAT.md](UAT.md).

## Start the Mapper Automatically

Enable the systemd user service:

```sh
systemctl --user enable --now shuttleprod.service
```

Or let the installer enable it while installing:

```sh
scripts/install.sh --configure-kdenlive
```

Stop it with:

```sh
systemctl --user stop shuttleprod.service
```

## Packaging Roadmap

The script installer is intentionally simple. A proper packaging pass should
add:

- RPM spec for Fedora and other RPM-based distributions;
- DEB packaging for Debian/Ubuntu;
- signed release archives containing the source, DKMS metadata, userspace
  binaries, profiles, and checksums;
- CI jobs that build packages from clean containers.
