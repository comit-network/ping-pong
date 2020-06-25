use anyhow::Result;
use log::{info, warn};
use torut::utils::{run_tor, AutoKillChild};

pub fn start_tor_instance() -> Result<()> {
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
    )?;
    let _child = AutoKillChild::new(child);

    info!("Tor instance started");
    Ok(())
}
