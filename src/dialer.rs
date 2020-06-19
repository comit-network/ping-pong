use crate::transport;
use anyhow::Result;
use async_std::task;
use futures::{future, prelude::*};
use libp2p::{
    identity,
    ping::{Ping, PingConfig},
    Multiaddr, PeerId, Swarm,
};
use std::task::{Context, Poll};

/// Entry point for the dialer sub-command.
pub fn run(addr: Multiaddr) -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let transport = transport::build(id_keys)?;
    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    Swarm::dial_addr(&mut swarm, addr).unwrap();

    task::block_on(future::poll_fn(move |cx: &mut Context| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => println!("{:?}", event),
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    }));

    Ok(())
}
