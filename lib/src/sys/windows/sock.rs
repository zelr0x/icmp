#![allow(non_camel_case_types)]
#![allow(unused)]

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::{be16, be32};

pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 23;

pub const IPV6_FLOWINFO_DEFAULT: u32 = 0;
pub const IPV6_SCOPE_ID_DEFAULT: u32 = 0;

// IPv4 address in BE.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct in_addr {
    pub bits: be32,
}

impl From<Ipv4Addr> for in_addr {
    fn from(value: Ipv4Addr) -> Self {
        in_addr {
            bits: be32::from_be_bytes(value.octets()).to_be(),
        }
    }
}

impl From<in_addr> for Ipv4Addr {
    fn from(value: in_addr) -> Self {
        Ipv4Addr::from_bits(u32::from_be(value.bits))
    }
}

impl From<in_addr> for be32 {
    fn from(value: in_addr) -> Self {
        value.bits
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct sockaddr_in {
    // AF_INET
    pub sin_family: u16,
    pub sin_port: be16,
    pub sin_addr: in_addr,
    sin_zero: [u8; 8],
}

impl sockaddr_in {
    pub fn new(addr: impl Into<in_addr>) -> Self {
        Self {
            sin_family: AF_INET,
            sin_addr: addr.into(),
            sin_port: 0_u16,
            sin_zero: [0; 8],
        }
    }
}

// IPv6 address in BE.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct in6_addr {
    pub octets: [u8; 16],
}

impl From<Ipv6Addr> for in6_addr {
    fn from(value: Ipv6Addr) -> Self {
        in6_addr {
            octets: value.octets(),
        }
    }
}

impl From<in6_addr> for Ipv6Addr {
    fn from(value: in6_addr) -> Self {
        Ipv6Addr::from_octets(value.octets)
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/ws2ipdef/ns-ws2ipdef-sockaddr_in6_lh
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in6 {
    // AF_INET6
    pub sin6_family: u16,
    pub sin6_port: be16,
    // IPv6 flow information
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    // IPv6 Scope ID
    pub sin6_scope_id: u32,
}

impl sockaddr_in6 {
    pub fn new(addr: impl Into<in6_addr>) -> Self {
        Self::new_ext(addr, 0, IPV6_FLOWINFO_DEFAULT, IPV6_SCOPE_ID_DEFAULT)
    }

    pub fn new_ext(addr: impl Into<in6_addr>, port: u16, flow_info: u32, scope_id: u32) -> Self {
        Self {
            sin6_family: AF_INET6,
            sin6_addr: addr.into(),
            sin6_port: port.to_be(),
            sin6_flowinfo: flow_info,
            sin6_scope_id: scope_id,
        }
    }
}
