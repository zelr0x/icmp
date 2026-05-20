#![allow(non_snake_case)]

pub(crate) mod apc;
pub(crate) mod ip;
pub(crate) mod sock;

use std::ffi::c_void;

use crate::{IpVer, be32};
use apc::io_apc_routine;
use ip::ip_option_information;
use sock::sockaddr_in6;

#[link(name = "IPHLPAPI")]
unsafe extern "system" {
    pub unsafe fn IcmpCreateFile() -> isize;

    pub unsafe fn Icmp6CreateFile() -> isize;

    // pub unsafe fn IcmpSendEcho(
    //     IcmpHandle: isize,
    //     DestinationAddress: be32,
    //     RequestData: *const u8,
    //     RequestSize: u16,
    //     RequestOptions: *mut ip_option_information, // optional
    //     ReplyBuffer: *mut u8,
    //     ReplySize: u32,
    //     Timeout: u32,
    // ) -> u32;

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

// Create a handle to an ICMP context.
pub fn icmp_handle(ver: IpVer) -> isize {
    match ver {
        IpVer::V4 => unsafe { IcmpCreateFile() },
        IpVer::V6 => unsafe { Icmp6CreateFile() },
    }
}
