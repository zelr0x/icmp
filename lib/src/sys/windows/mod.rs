#![allow(non_snake_case)]

pub(crate) mod apc;
pub(crate) mod ip;
pub(crate) mod sock;

use std::{
    ffi::c_void,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle},
};

use crate::{IpVer, be32};
use apc::io_apc_routine;
use ip::ip_option_information;
use sock::sockaddr_in6;

#[link(name = "IPHLPAPI")]
unsafe extern "system" {
    pub unsafe fn IcmpCreateFile() -> isize;

    pub unsafe fn IcmpCloseHandle(h: isize) -> bool;

    pub unsafe fn Icmp6CreateFile() -> isize;

    pub unsafe fn IcmpSendEcho2(
        IcmpHandle: isize,                 // required
        Event: isize,                      // optional
        ApcRoutine: *const io_apc_routine, // optional
        ApcContext: *const c_void,         // optional
        DestinationAddress: be32,
        RequestData: *const u8,
        RequestSize: u16,
        RequestOptions: *mut ip_option_information, // optional
        ReplyBuffer: *mut u8,
        ReplySize: u32,
        Timeout: u32,
    ) -> u32;

    pub unsafe fn Icmp6SendEcho2(
        IcmpHandle: isize,                 // required
        Event: isize,                      // optional
        ApcRoutine: *const io_apc_routine, // optional
        ApcContext: *const c_void,         // optional
        SourceAddress: *const sockaddr_in6,
        DestinationAddress: *const sockaddr_in6,
        RequestData: *const u8,
        RequestSize: u16,
        RequestOptions: *mut ip_option_information, // optional
        ReplyBuffer: *mut u8,
        ReplySize: u32,
        Timeout: u32,
    ) -> u32;
}

// A handle to an open ICMP context.
#[derive(Debug)]
pub struct IcmpHandle(isize);

impl IcmpHandle {
    pub fn create(ver: IpVer) -> Result<Self, io::Error> {
        match ver {
            IpVer::V4 => Self::create_ipv4(),
            IpVer::V6 => Self::create_ipv6(),
        }
    }

    pub fn create_ipv4() -> Result<Self, io::Error> {
        unsafe { Self::to_res(IcmpCreateFile()) }
    }

    pub fn create_ipv6() -> Result<Self, io::Error> {
        unsafe { Self::to_res(Icmp6CreateFile()) }
    }

    pub fn as_raw(&self) -> isize {
        self.0
    }

    fn to_res(h: isize) -> Result<Self, io::Error> {
        if h == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(h))
    }
}

impl Drop for IcmpHandle {
    fn drop(&mut self) {
        unsafe {
            _ = IcmpCloseHandle(self.0);
        }
    }
}

impl AsRawHandle for IcmpHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.0 as RawHandle
    }
}

impl IntoRawHandle for IcmpHandle {
    fn into_raw_handle(self) -> RawHandle {
        self.0 as RawHandle
    }
}

impl FromRawHandle for IcmpHandle {
    unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        IcmpHandle(handle as isize)
    }
}
