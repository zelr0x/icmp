#![allow(non_camel_case_types)]

use std::{ffi::c_int, net::{Ipv4Addr, Ipv6Addr}};

use crate::{be16, be32};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub type sa_family_t = u16;

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
))]
pub type sa_family_t = u8;

pub type socklen_t   = u32;

pub const AF_INET:  c_int  =  2;
pub const AF_INET6: c_int  = 10;

pub const SOCK_DGRAM: c_int = 2;  // Doesn't require CAP_NET_RAW/root
pub const SOCK_RAW:   c_int = 3;  // Requires CAP_NET_RAW/root

pub const IPPROTO_ICMP:   c_int =  1;
pub const IPPROTO_ICMPV6: c_int = 58;

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
#[derive(Debug, Clone, Copy)]
pub struct sockaddr {
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
    ))]
    sa_len: u8,

    sa_family: sa_family_t,
    sa_data: [u8; 14],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct sockaddr_in {
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
    ))]
    sin_len: u8,

    // AF_INET
    pub sin_family: sa_family_t,
    pub sin_port: be16,
    pub sin_addr: in_addr,
    sin_zero: [u8; 8],
}

impl sockaddr_in {
    pub fn new(addr: impl Into<in_addr>) -> Self {
        Self {
            #[cfg(any(
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "dragonfly",
                target_os = "macos",
                target_os = "ios",
            ))]
            sin_len: size_of::<sockaddr_in>() as u8,

            sin_family: AF_INET as sa_family_t,
            sin_addr: addr.into().into(),
            sin_port: 0,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in6 {
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
    ))]
    sin6_len: u8,
    // AF_INET6
    pub sin6_family: sa_family_t,
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
            #[cfg(any(
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "dragonfly",
                target_os = "macos",
                target_os = "ios",
            ))]
            sin6_len: size_of::<sockaddr_in6>() as u8,

            sin6_family: AF_INET6  as sa_family_t,
            sin6_addr: addr.into(),
            sin6_port: port.to_be(),
            sin6_flowinfo: flow_info,
            sin6_scope_id: scope_id,
        }
    }
}

#[link(name = "c")]
unsafe extern "C" {
    pub unsafe fn socket(domain: c_int, socket_type: c_int, protocol: c_int) -> c_int;

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
        addrlen: *mut socklen_t) -> isize;
}
