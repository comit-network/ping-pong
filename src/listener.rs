use anyhow::Result;
use async_std::task;
use futures::{future, prelude::*};
use libp2p::{
    identity,
    ping::{Ping, PingConfig},
    Multiaddr, PeerId, Swarm,
};
use std::task::{Context, Poll};

/// Entry point for the listener sub-command.
pub fn run(addr: Multiaddr) -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let transport = crate::build_transport(id_keys)?;
    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    Swarm::listen_on(&mut swarm, addr)?;

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => println!("{:?}", event),
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => {
                if !listening {
                    for addr in Swarm::listeners(&swarm) {
                        println!("Listening on {}", addr);
                        listening = true;
                    }
                }
                return Poll::Pending;
            }
        }
    }));

    Ok(())
}
