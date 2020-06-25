use libp2p::{
    multiaddr::{Onion3Addr, Protocol},
    Multiaddr,
};
use torut::onion::OnionAddressV3;

// We could do away with this file if libp2p and torut played nicely.
// TODO: Consider upstreaming things to facilitate this
// - torut: get_raw_bytes() could be public (giving us 35 bytes)

// Tor Onion v3 address: 32 public key bytes + 2 bytes of checksum + 0x03
//
// torut::onion::OnionAddressV3 :- 34 bytes (excl. the trailing '3' byte).
// libp2p::multiaddr::Protocol::Onion3(Onion3addr) :- 35 bytes plus port.

/// Wrap upstream onion address types.
pub struct OnionAddr<'a> {
    torut: OnionAddressV3,
    libp2p: Onion3Addr<'a>,
    port: u16,
}

impl OnionAddr<'_> {
    pub fn new(torut: OnionAddressV3, port: u16) -> Self {
        let mut buf = [0u8; 35];
        let tpk = torut.get_public_key();

        for (i, &byte) in tpk.as_bytes().iter().enumerate() {
            buf[i] = byte;
        }
        buf[34] = 3; // Onion address version.

        let libp2p = Onion3Addr::from((buf, port));

        OnionAddr {
            torut,
            libp2p,
            port,
        }
    }

    /// Onion address string:
    /// vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd.onion:1234
    pub fn address(&self) -> String {
        format!(
            "{}.onion:{}",
            self.torut.get_address_without_dot_onion(),
            self.port
        )
    }

    /// Onion multiaddr:
    /// /onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234
    pub fn multiaddr(self) -> Multiaddr {
        Multiaddr::empty().with(Protocol::Onion3(self.libp2p.clone()))
    }
}
