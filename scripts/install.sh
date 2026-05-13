#!/bin/sh

set -eu

repo=$(unset CDPATH; cd -- "$(dirname -- "$0")/.." && pwd)
package=hid-shuttlepro
version=0.1.0
prefix=/usr/local
install_dkms=1
install_userspace=1
install_user_service=1
configure_kdenlive=0

usage()
{
	cat <<EOF
Usage: scripts/install.sh [options]

Install the ShuttlePro v2 kernel driver and userspace tools.

Options:
  --prefix DIR          Install userspace binaries under DIR/bin (default: /usr/local)
  --no-dkms            Skip DKMS/kernel module installation
  --no-userspace       Skip Rust userspace build and binary installation
  --no-user-service    Skip systemd user service/profile installation
  --configure-kdenlive Install the Kdenlive profile and enable the mapper user service
  -h, --help           Show this help

Run as your normal desktop user. The script uses sudo for DKMS, udev, and
system binary installation.
EOF
}

while [ "$#" -gt 0 ]; do
	case "$1" in
	--prefix)
		[ "$#" -gt 1 ] || {
			echo "--prefix requires a directory" >&2
			exit 2
		}
		prefix=$2
		shift 2
		;;
	--no-dkms)
		install_dkms=0
		shift
		;;
	--no-userspace)
		install_userspace=0
		shift
		;;
	--no-user-service)
		install_user_service=0
		shift
		;;
	--configure-kdenlive)
		configure_kdenlive=1
		install_user_service=1
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		echo "unknown option: $1" >&2
		usage >&2
		exit 2
		;;
	esac
done

need_cmd()
{
	if ! command -v "$1" >/dev/null 2>&1; then
		echo "required command not found: $1" >&2
		exit 1
	fi
}

sudo_cmd()
{
	need_cmd sudo
	sudo "$@"
}

install_kernel_driver()
{
	need_cmd dkms

	echo "Installing DKMS module $package/$version"
	if dkms status "$package/$version" >/dev/null 2>&1; then
		sudo_cmd dkms remove "$package/$version" --all
	fi

	sudo_cmd dkms add "$repo"
	sudo_cmd dkms build "$package/$version"
	sudo_cmd dkms install "$package/$version"

	echo "Installing udev and module-load configuration"
	sudo_cmd install -m 0644 "$repo/99-hid-shuttlepro.rules" \
		/etc/udev/rules.d/99-hid-shuttlepro.rules
	sudo_cmd install -m 0644 "$repo/hid-shuttlepro.modules-load.conf" \
		/etc/modules-load.d/hid-shuttlepro.conf

	sudo_cmd udevadm control --reload-rules
	sudo_cmd udevadm trigger --subsystem-match=input
	sudo_cmd modprobe hid-shuttlepro
}

install_userspace_tools()
{
	need_cmd cargo

	echo "Building userspace tools"
	(
		cd "$repo/userspace"
		cargo build --release
	)

	echo "Installing userspace binaries to $prefix/bin"
	case "$prefix" in
	"$HOME" | "$HOME"/*)
		install -d -m 0755 "$prefix/bin"
		install -m 0755 "$repo/userspace/target/release/shuttleproctl" \
			"$prefix/bin/shuttleproctl"
		install -m 0755 "$repo/userspace/target/release/shuttleprod" \
			"$prefix/bin/shuttleprod"
		;;
	*)
		sudo_cmd install -d -m 0755 "$prefix/bin"
		sudo_cmd install -m 0755 "$repo/userspace/target/release/shuttleproctl" \
			"$prefix/bin/shuttleproctl"
		sudo_cmd install -m 0755 "$repo/userspace/target/release/shuttleprod" \
			"$prefix/bin/shuttleprod"
		;;
	esac
}

install_profiles_and_service()
{
	config_dir=${XDG_CONFIG_HOME:-"$HOME/.config"}
	profile_dir=$config_dir/shuttlepro/profiles
	profile_file=$profile_dir/kdenlive.toml
	service_dir=$config_dir/systemd/user
	service_file=$service_dir/shuttleprod.service

	echo "Installing user profiles to $profile_dir"
	install -d -m 0755 "$profile_dir"
	install -m 0644 "$repo/userspace/profiles/kdenlive.toml" "$profile_file"
	install -m 0644 "$repo/userspace/profiles/test.toml" "$profile_dir/test.toml"

	echo "Installing systemd user service to $service_file"
	install -d -m 0755 "$service_dir"
	{
		echo "[Unit]"
		echo "Description=Contour ShuttlePro v2 userspace profile mapper"
		echo "Documentation=https://github.com/Twix166/shuttle_pro_v2_driver"
		echo
		echo "[Service]"
		echo "ExecStart=$prefix/bin/shuttleprod --profile $profile_file"
		echo "Restart=on-failure"
		echo "RestartSec=2"
		echo
		echo "[Install]"
		echo "WantedBy=default.target"
	} > "$service_file"

	if command -v systemctl >/dev/null 2>&1; then
		systemctl --user daemon-reload || true
	fi
}

configure_kdenlive_profile()
{
	profile_file=${XDG_CONFIG_HOME:-"$HOME/.config"}/shuttlepro/profiles/kdenlive.toml

	if [ "$install_userspace" -eq 1 ]; then
		ctl=$prefix/bin/shuttleproctl
	else
		ctl=$(command -v shuttleproctl || true)
	fi

	if [ -z "${ctl:-}" ] || [ ! -x "$ctl" ]; then
		echo "shuttleproctl not found; cannot validate Kdenlive profile" >&2
		exit 1
	fi

	echo "Validating Kdenlive ShuttlePro profile"
	"$ctl" profile validate "$profile_file"

	if ! command -v kdenlive >/dev/null 2>&1; then
		echo "warning: kdenlive was not found on PATH; profile and service were still installed" >&2
	fi

	if ! command -v systemctl >/dev/null 2>&1; then
		echo "systemctl not found; cannot enable user service automatically" >&2
		return
	fi

	echo "Enabling ShuttlePro mapper user service for Kdenlive profile"
	systemctl --user daemon-reload || true
	systemctl --user enable --now shuttleprod.service
}

if [ "$(id -u)" -eq 0 ]; then
	echo "Run this installer as your normal user, not root." >&2
	echo "It will use sudo only for system-level installation steps." >&2
	exit 1
fi

if [ "$install_dkms" -eq 1 ]; then
	install_kernel_driver
fi

if [ "$install_userspace" -eq 1 ]; then
	install_userspace_tools
fi

if [ "$install_user_service" -eq 1 ]; then
	install_profiles_and_service
fi

if [ "$configure_kdenlive" -eq 1 ]; then
	configure_kdenlive_profile
fi

cat <<EOF

Install complete.

Suggested checks:
  shuttleproctl detect
  shuttleproctl tui
  shuttleprod --profile "\${XDG_CONFIG_HOME:-\$HOME/.config}/shuttlepro/profiles/kdenlive.toml" --dry-run --no-grab

To enable the user service:
  systemctl --user enable --now shuttleprod.service
EOF

if [ "$configure_kdenlive" -eq 1 ]; then
	cat <<EOF

Kdenlive integration was configured using the bundled ShuttlePro profile.
This does not rewrite Kdenlive's own shortcut files; it relies on Kdenlive's
documented default shortcuts. If you changed Kdenlive shortcuts, adjust:
  \${XDG_CONFIG_HOME:-\$HOME/.config}/shuttlepro/profiles/kdenlive.toml
EOF
fi
