use super::sum16;
use crate::{be16, be32};

pub mod icmp_type {
    pub const ICMP_ECHOREPLY: u8 = 0;
    pub const ICMP_ECHO: u8 = 8;
    // There are many more.
}

pub const ICMPHDR_SIZE: usize = size_of::<icmphdr>();
pub const ICMP6HDR_SIZE: usize = size_of::<icmp6hdr>();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Echo {
    pub id: be16,
    pub sequence: be16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Frag {
    pub __unused: u16,
    pub mtu: be16,
}

#[repr(C)]
pub union Un {
    pub echo: Echo,
    pub gateway: be32,
    pub frag: Frag,
}

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/icmp.h#L89
#[repr(C)]
pub struct icmphdr {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: sum16,
    pub un: Un,
}

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/icmpv6.h#L8
#[repr(C)]
pub struct icmp6hdr {
    // TODO: FIXME: implement
}
