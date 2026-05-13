# Hardening and Validation

This driver is small, but it runs in kernel space. Treat hardening as a set of
test lanes rather than a single tool.

## Automated Lanes

Run before every commit:

```sh
tests/ci.sh
tests/dkms-isolated.sh
```

Run the optional KUnit parser module on a local development machine with KUnit
enabled:

```sh
scripts/run-kunit.sh
```

The automated suite covers:

- shell syntax and shellcheck;
- parser regression tests for invalid reports, shuttle clamping, jog
  wraparound, and button masking;
- helper-script unit tests;
- license, DKMS, udev, and risk-notice metadata checks;
- kernel module build with `W=1`;
- optional `sparse` and Coccinelle checks when installed;
- isolated DKMS add/build.

## Kernel-Focused Frameworks

- KUnit is the preferred in-kernel unit test framework if this code is moved
  into the Linux kernel tree. The current standalone parser tests cover the
  same decode cases from userspace so they can run in ordinary GitHub Actions.
- kselftest-style scripts are appropriate for user-visible behavior and
  hardware validation. The existing `scripts/find-event.sh` and
  `scripts/test-events.sh` are the first layer of that approach.
- UHID is the right integration framework for virtual HID devices. A future
  privileged/self-hosted runner can create a virtual ShuttlePro report
  descriptor through `/dev/uhid`, inject raw reports, and assert evdev output.

## Sanitizer Lane

For deeper local testing, boot a debug kernel with:

- `CONFIG_KASAN=y` for memory safety bugs;
- `CONFIG_UBSAN=y` for undefined behavior;
- `CONFIG_KCSAN=y` for data race detection;
- `CONFIG_PROVE_LOCKING=y` and `CONFIG_LOCKDEP=y` for lock validation.

Then repeatedly load, use, unload, unplug, and replug the device while watching
`dmesg` for warnings, splats, sanitizer reports, or lockdep output.

## Hardware Regression Checklist

After changing `hid-shuttlepro.c`, verify on real hardware:

```sh
sudo rmmod hid_shuttlepro || true
sudo insmod ./hid-shuttlepro.ko
scripts/find-event.sh
sudo scripts/test-events.sh
```

Acceptance criteria:

- all 13 buttons emit named `BTN_TRIGGER_HAPPY*` press and release events;
- the shuttle ring emits `ABS_MISC` values from `-7` to `7` and returns to `0`;
- the jog wheel emits positive and negative `REL_DIAL` deltas;
- unplug/replug rebinds cleanly;
- repeated open/close and module reloads leave `dmesg` clean.
