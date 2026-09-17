//! This module handles connections and subscribers
//!
//! This includes a tcp and udp connections. As well
//! as subscribers to binary streams that are encoded
//! in the bc packets.
//!
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

mod bcconn;
mod bcsub;
mod discovery;
mod tcpsource;
mod udpsource;

pub(crate) use self::{
    bcconn::BcConnection, bcconn::*, bcsub::BcSubscription, discovery::Discovery,
    tcpsource::TcpSource, udpsource::UdpSource,
};

/// On Windows an ICMP "port unreachable" caused by an earlier send is reported
/// as WSAECONNRESET on the next receive of an *unconnected* UDP socket.
/// Neolink treats any receive error as fatal, so a single unreachable
/// destination (for example `p2p4.reolink.com`, which now resolves to
/// 127.0.0.1) aborts every pending discovery request or drops a running
/// stream. Linux never reports these errors, so we match that behaviour.
#[cfg(windows)]
pub(crate) fn disable_udp_connreset(socket: &UdpSocket) {
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::WSAIoctl;
    const SIO_UDP_CONNRESET: u32 = 0x9800_000C;
    let enable: u32 = 0;
    let mut returned: u32 = 0;
    // SAFETY: the socket handle is valid for the lifetime of `socket` and the
    // buffers passed are valid for the sizes given.
    let ret = unsafe {
        WSAIoctl(
            socket.as_raw_socket() as _,
            SIO_UDP_CONNRESET,
            &enable as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
            None,
        )
    };
    if ret != 0 {
        log::warn!("Unable to disable SIO_UDP_CONNRESET on a UDP socket");
    }
}

#[cfg(not(windows))]
pub(crate) fn disable_udp_connreset(_socket: &UdpSocket) {}

pub(crate) struct DiscoveryResult {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    client_id: i32,
    camera_id: i32,
}

impl DiscoveryResult {
    /// Get the address discovered
    pub(crate) fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }
}
