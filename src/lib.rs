mod cli;
mod onion;
mod transport;

pub use cli::Opt;
pub use onion::*;

use crate::transport::TokioTcpConfig;
use anyhow::{bail, Result};
use futures::{future, prelude::*};
use lazy_static::lazy_static;
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
use log::warn;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::net::TcpStream;
use tokio_socks::{tcp::Socks5Stream, IntoTargetAddr};
use torut::{
    control::UnauthenticatedConn,
    onion::TorSecretKeyV3,
    utils::{run_tor, AutoKillChild},
};

lazy_static! {
    /// The default TOR socks5 proxy address, `127.0.0.1:9050`.
    pub static ref TOR_PROXY_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    /// The default TOR Controller Protocol address, `127.0.0.1:9051`.
    pub static ref TOR_CP_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9051));
}

pub const PORT: u16 = 8007;

//pub const LOCAL_PORT: u16 = 7777;
//pub const ONION_PORT: u16 = 7;

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
pub async fn run_listener(local_addr: Multiaddr, local_port: u16, onion_port: u16) -> Result<()> {
    //
    // Run local Tor instance.
    //
    warn!("if Tor is already running attempting to start it again may hang ...");

    let child = run_tor(
        "/usr/bin/tor",
        &mut [
            "--CookieAuthentication",
            "1",
            "--defaults-torrc",
            "/usr/share/tor/tor-service-defaults-torrc",
            "-f",
            "/etc/tor/torrc",
        ]
        .iter(),
    )
    .expect("Starting tor filed");
    let _child = AutoKillChild::new(child);
    println!("Tor instance started");

    //
    // Get an authenticated connection to the Tor via the Tor Controller protocol.
    //

    let sock = TcpStream::connect(*TOR_CP_ADDR).await?;

    let mut utc = UnauthenticatedConn::new(sock);

    let info = match utc.load_protocol_info().await {
        Ok(info) => info,
        Err(_) => bail!("failed to load protocol info from Tor"),
    };
    let ad = info.make_auth_data()?.expect("failed to make auth data");

    if utc.authenticate(&ad).await.is_err() {
        bail!("failed to authenticate with Tor")
    }
    let mut ac = utc.into_authenticated().await;

    ac.set_async_event_handler(Some(|_| async move { Ok(()) }));

    ac.take_ownership().await.unwrap();

    //
    // Expose an onion service that re-directs to the echo server.
    //

    let key = TorSecretKeyV3::generate();
    ac.add_onion_v3(
        &key,
        false,
        false,
        false,
        None,
        &mut [(
            onion_port,
            SocketAddr::new(IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), local_port),
        )]
        .iter(),
    )
    .await
    .unwrap();

    let torut = key.public().get_onion_address();
    let onion = OnionAddr::from_torut(torut, PORT);

    println!(
        "Ping-pong onion service available at: \n\n\t{} \n\t{}\n",
        onion.address(),
        onion.multiaddr()
    );

    //
    // Start the ping-pong listener.
    //

    let config = PingConfig::new().with_keep_alive(true);
    let mut swarm = crate::build_swarm(config)?;

    Swarm::listen_on(&mut swarm, local_addr.clone())?;
    println!("Listening on {}", local_addr);

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

/// Connect to the Tor socks5 proxy socket.
pub async fn connect_tor_socks_proxy<'a>(dest: impl IntoTargetAddr<'a>) -> Result<TcpStream> {
    let sock = Socks5Stream::connect(*TOR_PROXY_ADDR, dest).await?;
    Ok(sock.into_inner())
}
