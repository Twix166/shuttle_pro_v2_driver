use libc::{c_int, timeval};
use std::fs::File;
use std::io::{self, ErrorKind};
use std::mem::{size_of, MaybeUninit};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_REL: u16 = 0x02;
pub const EV_ABS: u16 = 0x03;
pub const REL_DIAL: u16 = 0x07;
pub const ABS_MISC: u16 = 0x28;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InputEvent {
    pub time: timeval,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

pub struct EventDevice {
    file: File,
    grabbed: bool,
}

impl EventDevice {
    pub fn open(path: &Path, nonblocking: bool) -> io::Result<Self> {
        let flags =
            libc::O_RDONLY | libc::O_CLOEXEC | if nonblocking { libc::O_NONBLOCK } else { 0 };
        let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "path contains NUL byte"))?;
        let fd = unsafe { libc::open(c_path.as_ptr(), flags) };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            file: unsafe { File::from_raw_fd(fd) },
            grabbed: false,
        })
    }

    pub fn read_event(&self) -> io::Result<Option<InputEvent>> {
        let mut event = MaybeUninit::<InputEvent>::uninit();
        let size = size_of::<InputEvent>();
        let ret = unsafe { libc::read(self.file.as_raw_fd(), event.as_mut_ptr().cast(), size) };

        if ret < 0 {
            let err = io::Error::last_os_error();
            return if err.kind() == ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(err)
            };
        }

        if ret as usize != size {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "short input_event read",
            ));
        }

        Ok(Some(unsafe { event.assume_init() }))
    }

    pub fn grab(&mut self, enabled: bool) -> io::Result<()> {
        let value: c_int = if enabled { 1 } else { 0 };
        let ret = unsafe { libc::ioctl(self.file.as_raw_fd(), eviocgrab(), value) };

        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        self.grabbed = enabled;

        Ok(())
    }

    pub fn fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl Drop for EventDevice {
    fn drop(&mut self) {
        if self.grabbed {
            let _ = self.grab(false);
        }
    }
}

fn eviocgrab() -> libc::c_ulong {
    iow(b'E', 0x90, size_of::<c_int>())
}

pub(crate) fn iow(io_type: u8, nr: u8, size: usize) -> libc::c_ulong {
    const IOC_NRBITS: libc::c_ulong = 8;
    const IOC_TYPEBITS: libc::c_ulong = 8;
    const IOC_SIZEBITS: libc::c_ulong = 14;
    const IOC_NRSHIFT: libc::c_ulong = 0;
    const IOC_TYPESHIFT: libc::c_ulong = IOC_NRSHIFT + IOC_NRBITS;
    const IOC_SIZESHIFT: libc::c_ulong = IOC_TYPESHIFT + IOC_TYPEBITS;
    const IOC_DIRSHIFT: libc::c_ulong = IOC_SIZESHIFT + IOC_SIZEBITS;
    const IOC_WRITE: libc::c_ulong = 1;

    (IOC_WRITE << IOC_DIRSHIFT)
        | ((io_type as libc::c_ulong) << IOC_TYPESHIFT)
        | ((nr as libc::c_ulong) << IOC_NRSHIFT)
        | ((size as libc::c_ulong) << IOC_SIZESHIFT)
}

pub(crate) fn io(io_type: u8, nr: u8) -> libc::c_ulong {
    const IOC_NRBITS: libc::c_ulong = 8;
    const IOC_TYPEBITS: libc::c_ulong = 8;
    const IOC_SIZEBITS: libc::c_ulong = 14;
    const IOC_NRSHIFT: libc::c_ulong = 0;
    const IOC_TYPESHIFT: libc::c_ulong = IOC_NRSHIFT + IOC_NRBITS;
    const IOC_SIZESHIFT: libc::c_ulong = IOC_TYPESHIFT + IOC_TYPEBITS;
    const IOC_DIRSHIFT: libc::c_ulong = IOC_SIZESHIFT + IOC_SIZEBITS;

    (0 << IOC_DIRSHIFT)
        | ((io_type as libc::c_ulong) << IOC_TYPESHIFT)
        | ((nr as libc::c_ulong) << IOC_NRSHIFT)
        | (0 << IOC_SIZESHIFT)
}
