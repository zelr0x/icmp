use crate::{EchoRequest, EchoResult, EchoSession, Error, IpVer};
use std::{
    io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};

pub fn echo_session(req: EchoRequest) -> Result<EchoSession, Error> {
    EchoSession::new(req)
}

pub fn echo(session: &mut EchoSession, count: usize) -> impl Iterator<Item = EchoResult> {
    let mut count = count;
    std::iter::from_fn(move || {
        if count != 0 {
            count -= 1;
            Some(session.echo())
        } else {
            None
        }
    })
}

// Resolves a hostname to the first IP address of a given verison.
pub fn lookup_host(hostname: &str, ver: IpVer) -> Result<IpAddr, Error> {
    let addrs = if !hostname.contains(":") {
        (hostname, 0).to_socket_addrs()
    } else {
        hostname.to_socket_addrs()
    };
    let ips = to_ips(addrs)?;
    first_ip_with_ver(ips, ver).ok_or(Error::HostnameResolutionFailed)
}

fn to_ips(
    iter: io::Result<impl Iterator<Item = SocketAddr>>,
) -> Result<impl Iterator<Item = IpAddr>, Error> {
    iter.map(|r| r.map(|x| x.ip()))
        .map_err(|_e| Error::HostnameResolutionFailed)
}

fn first_ip_with_ver(mut ips: impl Iterator<Item = IpAddr>, ver: IpVer) -> Option<IpAddr> {
    if ver.is_ipv4() {
        ips.find(|&ip| ip.is_ipv4())
    } else {
        ips.find(|&ip| ip.is_ipv6())
    }
}
