use anyhow::Result;
use futures::{future, prelude::*};
use libp2p::{ping::PingConfig, Multiaddr, Swarm};
use std::task::{Context, Poll};
use std::time::Duration;

/// Entry point for the dialer sub-command.
pub async fn run(addr: Multiaddr) -> Result<()> {
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
