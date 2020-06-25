mod cli;
mod transport;

pub use cli::Opt;

use crate::transport::TokioTcpConfig;
use anyhow::Result;
use futures::{future, prelude::*};
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
    swarm::SwarmBuilder,
    yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// Entry point to run the ping-pong application as a dialer.
pub async fn run_dialer(addr: Multiaddr) -> Result<()> {
    let config = PingConfig::new()
        .with_keep_alive(true)
        .with_interval(Duration::from_secs(1));
    let mut swarm = crate::build_swarm(config)?;

    Swarm::dial_addr(&mut swarm, addr).unwrap();

    future::poll_fn(move |cx: &mut Context| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => println!("{:?}", event),
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    })
    .await;

    Ok(())
}

/// Entry point to run the ping-pong application as a listener.
pub async fn run_listener(addr: Multiaddr) -> Result<()> {
    let config = PingConfig::new().with_keep_alive(true);
    let mut swarm = crate::build_swarm(config)?;

    Swarm::listen_on(&mut swarm, addr.clone())?;
    println!("Listening on {}", addr);

    future::poll_fn(move |cx: &mut Context| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => println!("{:?}", event),
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    })
    .await;

    Ok(())
}

/// Build a libp2p swarm (also called a switch).
pub fn build_swarm(config: PingConfig) -> Result<Swarm<Ping>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let transport = crate::build_transport(id_keys)?;
    let behaviour = Ping::new(config);

    let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
        .executor(Box::new(TokioExecutor))
        .build();

    Ok(swarm)
}

struct TokioExecutor;

impl libp2p::core::Executor for TokioExecutor {
    fn exec(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        tokio::spawn(future);
    }
}

/// Builds a libp2p transport with the following features:
/// - TCp connectivity
/// - DNS name resolution
/// - Authentication via secio
/// - Multiplexing via yamux or mplex
pub fn build_transport(keypair: identity::Keypair) -> anyhow::Result<PingPongTransport> {
    let transport = TokioTcpConfig::new().nodelay(true);
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

/// libp2p `Transport` for the ping-pong application.
pub type PingPongTransport = Boxed<
    (PeerId, StreamMuxerBox),
    TransportTimeoutError<
        EitherError<
            EitherError<DnsErr<io::Error>, UpgradeError<SecioError>>,
            UpgradeError<EitherError<io::Error, io::Error>>,
        >,
    >,
>;
