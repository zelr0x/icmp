#![allow(non_camel_case_types)]

use std::ffi::c_void;

use crate::sys::windows::sock::sockaddr_in6;

use super::sock::in_addr;

pub const ICMP_ECHO_REPLY_SIZE: usize = size_of::<icmp_echo_reply>();
pub const ICMP6_ECHO_REPLY_SIZE: usize = size_of::<icmpv6_echo_reply>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ip_option_information {
    pub Ttl: u8,
    pub Tos: u8,
    pub Flags: u8,
    pub OptionsSize: u8,
    pub OptionsData: *const u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct icmp_echo_reply {
    pub Address: in_addr,
    pub Status: u32,
    pub RoundTripTime: u32,
    pub DataSize: u16,
    pub Reserved: u16,
    pub Data: *const c_void,
    pub Options: ip_option_information,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct icmpv6_echo_reply {
    pub Address: sockaddr_in6,
    pub Status: u32,
    pub RoundTripTime: u32,
    pub DataSize: u16,
    pub Reserved: u16,
    pub Data: *const c_void,
    pub Options: ip_option_information,
}
