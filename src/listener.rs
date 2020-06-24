use anyhow::Result;
use futures::{future, prelude::*};
use libp2p::{ping::PingConfig, Multiaddr, Swarm};
use std::task::{Context, Poll};

/// Entry point for the listener sub-command.
pub async fn run(addr: Multiaddr) -> Result<()> {
    let config = PingConfig::new().with_keep_alive(true);
    let mut swarm = crate::build_swarm(config)?;

    Swarm::listen_on(&mut swarm, addr)?;

    let mut listening = false;
    future::poll_fn(move |cx: &mut Context| loop {
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
    })
    .await;

    Ok(())
}
