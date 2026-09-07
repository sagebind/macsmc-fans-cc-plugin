#![allow(unused)]

use nix::{errno::Errno, ioctl_none, libc::ioctl, request_code_read, request_code_write};
use std::{mem::size_of, os::fd::RawFd};

const MAGIC: u8 = 0xec;
const MAGIC_READ: u8 = MAGIC + 3;
const MAGIC_WRITE: u8 = MAGIC + 4;

macro_rules! ioctl_read_int {
    ($name:ident, $id:expr, $seq:expr) => {
        pub unsafe fn $name(fd: RawFd, data_ptr: *mut i32) -> ::nix::Result<i32> {
            let request_code = request_code_read!($id, $seq, size_of::<*mut i32>());
            Errno::result(unsafe { ioctl(fd, request_code, data_ptr) })
        }
    };
}

macro_rules! ioctl_write_int {
    ($name:ident, $id:expr, $seq:expr) => {
        pub unsafe fn $name(fd: RawFd, data_ptr: *const i32) -> ::nix::Result<i32> {
            let request_code = request_code_write!($id, $seq, size_of::<*const i32>());
            Errno::result(unsafe { ioctl(fd, request_code, data_ptr) })
        }
    };
}

ioctl_read_int!(uw_hwcheck, MAGIC, 0x06);

ioctl_read_int!(r_uw_fanspeed, MAGIC_READ, 0x10);
ioctl_read_int!(r_uw_fanspeed2, MAGIC_READ, 0x11);
ioctl_read_int!(r_uw_fan_temp, MAGIC_READ, 0x12);
ioctl_read_int!(r_uw_fan_temp2, MAGIC_READ, 0x13);
ioctl_read_int!(r_uw_mode, MAGIC_READ, 0x14);
ioctl_read_int!(r_uw_mode_enable, MAGIC_READ, 0x15);
ioctl_read_int!(r_uw_fans_off_available, MAGIC_READ, 0x16);
ioctl_read_int!(r_uw_fans_min_speed, MAGIC_READ, 0x17);
ioctl_read_int!(r_uw_tdp0, MAGIC_READ, 0x18);
ioctl_read_int!(r_uw_tdp1, MAGIC_READ, 0x19);
ioctl_read_int!(r_uw_tdp2, MAGIC_READ, 0x1a);
ioctl_read_int!(r_uw_tdp0_min, MAGIC_READ, 0x1b);
ioctl_read_int!(r_uw_tdp1_min, MAGIC_READ, 0x1c);
ioctl_read_int!(r_uw_tdp2_min, MAGIC_READ, 0x1d);
ioctl_read_int!(r_uw_tdp0_max, MAGIC_READ, 0x1e);
ioctl_read_int!(r_uw_tdp1_max, MAGIC_READ, 0x1f);
ioctl_read_int!(r_uw_tdp2_max, MAGIC_READ, 0x20);

ioctl_write_int!(w_uw_fanspeed, MAGIC_WRITE, 0x10);
ioctl_write_int!(w_uw_fanspeed2, MAGIC_WRITE, 0x11);
ioctl_write_int!(w_uw_mode, MAGIC_WRITE, 0x12);
ioctl_write_int!(w_uw_mode_enable, MAGIC_WRITE, 0x13);
ioctl_none!(w_uw_fanauto, MAGIC_WRITE, 0x14);
ioctl_write_int!(w_uw_tdp0, MAGIC_WRITE, 0x15);
ioctl_write_int!(w_uw_tdp1, MAGIC_WRITE, 0x16);
ioctl_write_int!(w_uw_tdp2, MAGIC_WRITE, 0x17);
ioctl_write_int!(w_uw_perf_prof, MAGIC_WRITE, 0x18);
