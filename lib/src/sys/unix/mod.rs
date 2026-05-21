#![allow(non_camel_case_types)]

pub(crate) mod icmp;
pub(crate) mod sock;

use crate::{
    IpVer,
    sys::unix::sock::{AF_INET, AF_INET6, IPPROTO_ICMP, SOCK_DGRAM, socket}
};

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

pub fn icmp_dgram_socket(ver: IpVer) -> i32 {
    let domain = match ver {
        IpVer::V4 => AF_INET,
        IpVer::V6 => AF_INET6,
    };
    unsafe {
        socket(domain, SOCK_DGRAM, IPPROTO_ICMP)
    }
}
