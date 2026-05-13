use crate::input::{io, iow, EV_KEY, EV_SYN};
use crate::keys::{all_keys, Key, KeyChord};
use libc::input_id;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::mem::size_of;
use std::os::fd::AsRawFd;

const SYN_REPORT: u16 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
struct UInputUserDev {
    name: [u8; 80],
    id: input_id,
    ff_effects_max: u32,
    absmax: [i32; 64],
    absmin: [i32; 64],
    absfuzz: [i32; 64],
    absflat: [i32; 64],
}

pub struct VirtualKeyboard {
    file: File,
}

impl VirtualKeyboard {
    pub fn create(name: &str, chords: &[KeyChord]) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")?;

        ioctl_set(file.as_raw_fd(), ui_set_evbit(), EV_KEY)?;
        ioctl_set(file.as_raw_fd(), ui_set_evbit(), EV_SYN)?;

        for key in all_keys(chords) {
            ioctl_set(file.as_raw_fd(), ui_set_keybit(), key.code)?;
        }

        let mut dev = UInputUserDev {
            name: [0; 80],
            id: input_id {
                bustype: 0x03,
                vendor: 0x0b33,
                product: 0x0030,
                version: 1,
            },
            ff_effects_max: 0,
            absmax: [0; 64],
            absmin: [0; 64],
            absfuzz: [0; 64],
            absflat: [0; 64],
        };

        let bytes = name.as_bytes();
        let len = bytes.len().min(dev.name.len() - 1);
        dev.name[..len].copy_from_slice(&bytes[..len]);

        let dev_bytes = unsafe {
            std::slice::from_raw_parts(
                (&dev as *const UInputUserDev).cast::<u8>(),
                size_of::<UInputUserDev>(),
            )
        };
        file.write_all(dev_bytes)?;

        let ret = unsafe { libc::ioctl(file.as_raw_fd(), ui_dev_create()) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { file })
    }

    pub fn tap_chord(&mut self, chord: &KeyChord) -> io::Result<()> {
        for key in &chord.modifiers {
            self.emit_key(*key, 1)?;
        }

        self.emit_key(chord.key, 1)?;
        self.emit_key(chord.key, 0)?;

        for key in chord.modifiers.iter().rev() {
            self.emit_key(*key, 0)?;
        }

        self.sync()
    }

    fn emit_key(&mut self, key: Key, value: i32) -> io::Result<()> {
        self.emit(EV_KEY, key.code, value)
    }

    fn sync(&mut self) -> io::Result<()> {
        self.emit(EV_SYN, SYN_REPORT, 0)
    }

    fn emit(&mut self, event_type: u16, code: u16, value: i32) -> io::Result<()> {
        let event = libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: event_type,
            code,
            value,
        };
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&event as *const libc::input_event).cast::<u8>(),
                size_of::<libc::input_event>(),
            )
        };

        self.file.write_all(bytes)
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        unsafe {
            libc::ioctl(self.file.as_raw_fd(), ui_dev_destroy());
        }
    }
}

fn ioctl_set(fd: i32, request: libc::c_ulong, value: u16) -> io::Result<()> {
    let ret = unsafe { libc::ioctl(fd, request, libc::c_int::from(value)) };

    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

fn ui_dev_create() -> libc::c_ulong {
    io(b'U', 1)
}

fn ui_dev_destroy() -> libc::c_ulong {
    io(b'U', 2)
}

fn ui_set_evbit() -> libc::c_ulong {
    iow(b'U', 100, size_of::<libc::c_int>())
}

fn ui_set_keybit() -> libc::c_ulong {
    iow(b'U', 101, size_of::<libc::c_int>())
}
