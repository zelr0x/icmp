use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
    vec,
};

use crate::Error;

#[cfg(windows)]
use std::ffi::c_void;

#[cfg(windows)]
use crate::sys::windows::{
    self,
    apc::io_apc_routine,
    ip::{
        ICMP_ECHO_REPLY_SIZE, ICMP6_ECHO_REPLY_SIZE, icmp_echo_reply, icmpv6_echo_reply,
        ip_option_information,
    },
    sock::{in_addr, sockaddr_in6},
};

#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
use crate::sys::unix::{
    self,
    icmp::{Echo, ICMPHDR_SIZE, Un, icmp_type, icmphdr},
    sock::{recvfrom, sendto, sockaddr, sockaddr_in, socklen_t},
    sum16,
};

#[derive(Debug, Clone)]
pub struct EchoRequest<'a> {
    pub addr: IpAddr,
    pub data: &'a [u8],
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct EchoReply {
    pub addr: IpAddr,
    pub rtt: Duration,
    pub ttl: u8,
    pub data: Box<[u8]>,
}

#[derive(Debug)]
pub enum EchoError {
    NetApiErr(io::Error), // FIXME: GetLastError()
    IcmpErr,
}

pub type EchoResult = Result<EchoReply, EchoError>;

#[cfg(windows)]
#[derive(Debug)]
pub struct EchoSession<'a> {
    addr: IpAddr,
    request_data: &'a [u8],
    request_size: u16,
    handle: isize,
    event: isize,
    apc_routine: *const io_apc_routine,
    apc_context: *const c_void,
    opts: *mut ip_option_information,
    reply_buf: Vec<u8>,
    reply_size: u32,
    timeout: u32,
}

#[cfg(windows)]
impl<'a> EchoSession<'a> {
    pub fn new(req: EchoRequest<'a>) -> Result<Self, Error> {
        let request_data = req.data;
        let request_size = request_data.len();
        let icmp_echo_reply_size: usize = Self::get_icmp_echo_reply_size(req.addr);
        if request_size > u16::MAX as usize - icmp_echo_reply_size {
            return Err(Error::RequestDataTooBig);
        }
        let request_size = request_size as u16;

        let reply_size = icmp_echo_reply_size as u32 + request_size as u32 + 1024;
        let reply_buf = vec![0u8; reply_size as usize];

        let ver = req.addr.into();
        let handle = windows::icmp_handle(ver);

        let event = 0;
        let apc_routine = std::ptr::null();
        let apc_context = std::ptr::null();

        let opts: *mut ip_option_information = std::ptr::null_mut();

        let timeout = req.timeout.as_millis() as u32;

        let res = Self {
            addr: req.addr,
            request_data: req.data,
            request_size,
            handle,
            event,
            apc_routine,
            apc_context,
            opts,
            reply_buf,
            reply_size,
            timeout,
        };
        Ok(res)
    }

    // Sends a burst of `count` ICMP echo messages and calls handle on the results.
    // Either returns an error or calls handle exactly count times.
    pub fn echo(&mut self) -> EchoResult {
        // TODO: FIXME: this is a unicast-only implementation.
        // reply_buf can contain more than one reply when used with multicast,
        // anycast as well, probably -- check.

        // TODO: Make request_data a parameter?

        let request_data = self.request_data.as_ptr();
        let off = Self::get_icmp_echo_reply_size(self.addr);
        match self.addr {
            IpAddr::V4(addr) => {
                let dest_addr: in_addr = addr.into();
                let dest_addr: u32 = dest_addr.into();
                let got_replies = unsafe {
                    windows::IcmpSendEcho2(
                        self.handle,
                        self.event,
                        self.apc_routine,
                        self.apc_context,
                        dest_addr,
                        request_data,
                        self.request_size,
                        self.opts,
                        self.reply_buf.as_mut_ptr(),
                        self.reply_size,
                        self.timeout,
                    )
                };
                // TODO: add ERROR_IO_PENDING const and check it for async variant
                if got_replies == 0 {
                    return Err(EchoError::NetApiErr(io::Error::last_os_error()));
                }
                // TODO: ensure read_analigned is the way to go or &* is enough
                let reply = unsafe {
                    std::ptr::read_unaligned(self.reply_buf.as_ptr() as *const icmp_echo_reply)
                };
                let reply_addr: Ipv4Addr = reply.Address.into();
                let reply_data = self.reply_buf[off..off + reply.DataSize as usize]
                    .to_vec()
                    .into_boxed_slice();
                let res = EchoReply {
                    addr: reply_addr.into(),
                    rtt: Duration::from_millis(reply.RoundTripTime as u64),
                    ttl: reply.Options.Ttl,
                    data: reply_data,
                };
                Ok(res)
            }
            // TODO: test
            IpAddr::V6(addr) => {
                let dest_addr: *const sockaddr_in6 = &sockaddr_in6::new(addr);
                let got_replies = unsafe {
                    windows::Icmp6SendEcho2(
                        self.handle,
                        self.event,
                        self.apc_routine,
                        self.apc_context,
                        std::ptr::null(), // auto-detect source address
                        dest_addr,
                        request_data,
                        self.request_size,
                        self.opts,
                        self.reply_buf.as_mut_ptr(),
                        self.reply_size,
                        self.timeout,
                    )
                };
                if got_replies == 0 {
                    return Err(EchoError::IcmpErr);
                }
                // TODO: ensure read_analigned is the way to go or &* is enough
                let reply = unsafe {
                    std::ptr::read_unaligned(self.reply_buf.as_ptr() as *const icmpv6_echo_reply)
                };
                let reply_addr: Ipv6Addr = reply.Address.sin6_addr.into();
                let reply_data = self.reply_buf[off..off + reply.DataSize as usize]
                    .to_vec()
                    .into_boxed_slice();
                let res = EchoReply {
                    addr: reply_addr.into(),
                    rtt: Duration::from_millis(reply.RoundTripTime as u64),
                    ttl: reply.Options.Ttl,
                    data: reply_data,
                };
                Ok(res)
            }
        }
    }

    const fn get_icmp_echo_reply_size(addr: IpAddr) -> usize {
        match addr {
            IpAddr::V4(_) => ICMP_ECHO_REPLY_SIZE,
            IpAddr::V6(_) => ICMP6_ECHO_REPLY_SIZE,
        }
    }
}

#[cfg(unix)]
#[derive(Debug)]
pub struct EchoSession<'a> {
    addr: IpAddr,
    echo: Echo,
    request_data: &'a [u8],
    request_size: u16,
    sockfd: i32,
    reply_buf: Vec<u8>,
}

#[cfg(unix)]
impl<'a> EchoSession<'a> {
    pub fn new(req: EchoRequest<'a>) -> Result<Self, Error> {
        let id = u16::try_from(std::process::id() & 0xFFFF).unwrap().to_be();
        let echo = Echo { id, sequence: 0 };

        let request_data = req.data;
        let request_size = request_data.len();
        // TODO: FIXME: insufficient check, request_size must be even smaller
        if request_size as u16 > u16::MAX {
            return Err(Error::RequestDataTooBig);
        }
        let request_size = request_size as u16;

        let ver = req.addr.into();
        let sockfd = unix::icmp_dgram_socket(ver);
        if sockfd == -1 {
            return Err(Error::SocketOpenFailed(io::Error::last_os_error()));
        }

        // TODO: FIXME: hardcoded reply buffer size
        let reply_buf = vec![0u8; 2048];

        let res = Self {
            addr: req.addr.into(),
            sockfd,
            request_data,
            request_size,
            reply_buf: reply_buf,
            echo,
        };
        Ok(res)
    }

    pub fn echo(&mut self) -> EchoResult {
        match self.addr {
            IpAddr::V4(addr) => {
                // TODO: Either move icmphdr and packet to session or leave here
                // but make request_data a parameter (in which case change
                // for windows as well, obviuosly).

                let echo = {
                    let mut echo = self.echo;
                    echo.sequence += 1;
                    echo
                };
                let un = Un { echo };
                let mut hdr: icmphdr = icmphdr {
                    icmp_type: icmp_type::ICMP_ECHO,
                    code: 0,
                    checksum: sum16::default(),
                    un,
                };

                let mut packet = Vec::with_capacity(ICMPHDR_SIZE + self.request_size as usize);
                packet.extend_from_slice(unsafe {
                    std::slice::from_raw_parts(&hdr as *const icmphdr as *const u8, ICMPHDR_SIZE)
                });
                packet.extend_from_slice(self.request_data);

                // Compute checksum
                hdr.checksum = unix::sum16::new(&packet);
                // Update header with checksum
                packet[..ICMPHDR_SIZE].copy_from_slice(unsafe {
                    std::slice::from_raw_parts(&hdr as *const icmphdr as *const u8, ICMPHDR_SIZE)
                });

                let dest_addr = sockaddr_in::new(addr);
                let start = Instant::now();
                let ret = unsafe {
                    sendto(
                        self.sockfd,
                        packet.as_ptr() as *const _,
                        packet.len(),
                        0,
                        &dest_addr as *const sockaddr_in as *const sockaddr,
                        size_of::<sockaddr_in>() as u32,
                    )
                };
                if ret < 0 {
                    return Err(EchoError::NetApiErr(io::Error::last_os_error()));
                }

                let mut reply_addr: sockaddr_in = sockaddr_in::default();
                let mut reply_addr_len = size_of::<sockaddr_in>() as socklen_t;
                let n = unsafe {
                    recvfrom(
                        self.sockfd,
                        self.reply_buf.as_mut_ptr() as *mut _,
                        self.reply_buf.len(),
                        0,
                        &mut reply_addr as *mut sockaddr_in as *mut sockaddr,
                        &mut reply_addr_len,
                    )
                };
                if n < 0 {
                    return Err(EchoError::NetApiErr(io::Error::last_os_error()));
                }
                let end = Instant::now();
                let rtt = end.duration_since(start);

                // TODO: ensure read_analigned is the way to go or &* is enough
                let reply_hdr: icmphdr =
                    unsafe { std::ptr::read_unaligned(self.reply_buf.as_ptr() as *const icmphdr) };
                let reply_addr: Ipv4Addr = reply_addr.sin_addr.into();
                let reply_data = self.reply_buf[ICMPHDR_SIZE..n as usize]
                    .to_vec()
                    .into_boxed_slice();
                let res = EchoReply {
                    addr: reply_addr.into(),
                    rtt,
                    ttl: 0, // TODO: FIXME: find it with setsockopt IP_RECVTTL, it is another rabbit hole
                    data: reply_data,
                };
                Ok(res)
            }
            IpAddr::V6(addr) => {
                unimplemented!()
            }
        }
    }
}
