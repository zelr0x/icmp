#![allow(non_camel_case_types)]

pub(crate) mod icmp;
pub(crate) mod sock;

use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::{ffi::c_int, io};

use crate::{
    IpVer,
    sys::unix::sock::{AF_INET, AF_INET6, IPPROTO_ICMP, SOCK_DGRAM, sockaddr, socklen_t},
};

#[link(name = "c")]
unsafe extern "C" {
    pub unsafe fn socket(domain: c_int, socket_type: c_int, protocol: c_int) -> c_int;

    pub unsafe fn close(fd: c_int) -> c_int;

    pub unsafe fn sendto(
        sockfd: c_int,
        buf: *const u8,
        len: usize,
        flags: c_int,
        dest_addr: *const sockaddr,
        addrlen: socklen_t,
    ) -> isize;

    pub unsafe fn recvfrom(
        sockfd: c_int,
        buf: *mut u8,
        len: usize,
        flags: c_int,
        src_addr: *mut sockaddr,
        addrlen: *mut socklen_t,
    ) -> isize;
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct sum16(u16);

impl sum16 {
    // TODO: verify algorithm.
    pub fn new(buf: &[u8]) -> Self {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i + 1 < buf.len() {
            let word = u16::from_be_bytes([buf[i], buf[i + 1]]) as u32;
            sum = sum.wrapping_add(word);
            i += 2;
        }
        if i < buf.len() {
            sum = sum.wrapping_add((buf[i] as u32) << 8);
        }
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        Self(!(sum as u16))
    }
}

// A descriptor of an open ICMP (SOCK_DGRAM + IPPROTO_ICMP) socket.
#[derive(Debug)]
pub struct IcmpSocket(i32);

impl IcmpSocket {
    pub fn open(ver: IpVer) -> Result<Self, io::Error> {
        match ver {
            IpVer::V4 => Self::open_ipv4(),
            IpVer::V6 => Self::open_ipv6(),
        }
    }

    pub fn open_ipv4() -> Result<Self, io::Error> {
        unsafe { Self::do_open(AF_INET) }
    }

    pub fn open_ipv6() -> Result<Self, io::Error> {
        unsafe { Self::do_open(AF_INET6) }
    }

    pub fn as_raw(&self) -> i32 {
        self.0
    }

    fn do_open(domain: c_int) -> Result<Self, io::Error> {
        let fd = unsafe { socket(domain, SOCK_DGRAM, IPPROTO_ICMP) };
        if fd == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(fd))
    }
}

impl Drop for IcmpSocket {
    fn drop(&mut self) {
        unsafe {
            _ = close(self.0);
        }
    }
}

impl AsRawFd for IcmpSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for IcmpSocket {
    fn into_raw_fd(self) -> RawFd {
        self.0
    }
}

impl FromRawFd for IcmpSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        IcmpSocket(fd)
    }
}
