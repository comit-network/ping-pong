pub mod dialer;
pub mod listener;

use anyhow::Result;
use libp2p::{
    core::{
        either::EitherError,
        muxing::StreamMuxerBox,
        transport::{boxed::Boxed, timeout::TransportTimeoutError},
        upgrade::{SelectUpgrade, Version},
        UpgradeError,
    },
    dns::{DnsConfig, DnsErr},
    identity,
    mplex::MplexConfig,
    ping::{Ping, PingConfig},
    secio::{SecioConfig, SecioError},
    tcp::TcpConfig,
    yamux, PeerId, Swarm, Transport,
};
use std::{io, time::Duration};

pub type PingPongTransport = Boxed<
    (PeerId, StreamMuxerBox),
    TransportTimeoutError<
        EitherError<
            EitherError<DnsErr<io::Error>, UpgradeError<SecioError>>,
            UpgradeError<EitherError<io::Error, io::Error>>,
        >,
    >,
>;

/// Builds a libp2p transport with the following features:
/// - TcpConnection
/// - DNS name resolution
/// - authentication via secio
/// - multiplexing via yamux or mplex
pub fn build_transport(keypair: identity::Keypair) -> anyhow::Result<PingPongTransport> {
    let transport = TcpConfig::new().nodelay(true);
    let transport = DnsConfig::new(transport)?;

    let transport = transport
        .upgrade(Version::V1)
        .authenticate(SecioConfig::new(keypair))
        .multiplex(SelectUpgrade::new(
            yamux::Config::default(),
            MplexConfig::new(),
        ))
        .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
        .timeout(Duration::from_secs(20))
        .boxed();

    Ok(transport)
}

pub fn build_swarm(config: PingConfig) -> Result<Swarm<Ping>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let transport = crate::build_transport(id_keys)?;
    let behaviour = Ping::new(config);

    let swarm = Swarm::new(transport, behaviour, peer_id);

    Ok(swarm)
}
