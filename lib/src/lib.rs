mod echo;
mod run;
mod sys;

use std::{io, net::IpAddr};

pub use echo::{EchoError, EchoReply, EchoRequest, EchoResult, EchoSession};
pub use run::*;

// The following two types are aliases on purpose for flexibility.
// Converting it to a newtype is an option but care must be taken either way.

// u16 that must be htons-ed before storing in a variable of this type.
#[allow(non_camel_case_types)]
pub(crate) type be16 = u16;
// u32 that must be htons-ed before storing in a variable of this type.
#[allow(non_camel_case_types)]
pub(crate) type be32 = u32;

#[derive(Debug, Clone)]
pub enum Host {
    Ip(IpAddr),
    Hostname(String),
}

// TODO: better errors
#[derive(Debug)]
pub enum Error {
    HostnameResolutionFailed,
    IoErr(io::Error),
    RequestDataTooBig,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IpVer {
    V4,
    V6,
}

impl IpVer {
    pub fn is_ipv4(self) -> bool {
        matches!(self, IpVer::V4)
    }

    pub fn is_ipv6(self) -> bool {
        matches!(self, IpVer::V6)
    }
}

impl From<IpAddr> for IpVer {
    fn from(value: IpAddr) -> Self {
        if value.is_ipv4() {
            IpVer::V4
        } else {
            IpVer::V6
        }
    }
}
