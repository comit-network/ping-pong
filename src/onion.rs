use anyhow::{anyhow, Result};
use libp2p::{
    multiaddr::{Onion3Addr, Protocol},
    Multiaddr,
};
use torut::onion::{OnionAddressV3, TorPublicKeyV3};

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
    /// Create an onion address from a torut onion address.
    pub fn from_torut(torut: OnionAddressV3, port: u16) -> Self {
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

    /// Crate an onion address from a libp2p Multiaddr.
    pub fn from_multiaddr(mut multi: Multiaddr) -> Result<Self> {
        match multi.pop() {
            Some(Protocol::Onion3(onion)) => {
                let mut buf = [0u8; 32];
                let bytes = onion.hash();
                for (i, &byte) in bytes.iter().enumerate() {
                    // pub key is the first 32 bytes.
                    if i == 32 {
                        break;
                    }
                    buf[i] = byte;
                }
                let port = onion.port();

                let key = TorPublicKeyV3::from_bytes(&buf)?;
                let torut = OnionAddressV3::from(&key);

                Ok(OnionAddr {
                    torut,
                    libp2p: onion,
                    port,
                })
            }
            _ => Err(anyhow!("not an onion v3 multiaddr")),
        }
    }

    /// vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd.onion:1234
    pub fn address(&self) -> String {
        format!(
            "{}.onion:{}",
            self.torut.get_address_without_dot_onion(),
            self.port
        )
    }

    /// /onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234
    pub fn multiaddr(self) -> Multiaddr {
        Multiaddr::empty().with(Protocol::Onion3(self.libp2p.clone()))
    }
}
