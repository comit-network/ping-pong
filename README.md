# Project Harpocrates - Ping-Pong

Basic TCP peer to peer connectivity [1] using libp2p over the Tor network.

## ping-pong v0.2

From v0.2 there are a number of steps one needs to take in order to
run ping-pong.

0. Install Tor
1. Configure an onion service using the Tor run file. You can use
`./torrc` and run `tor` using `sudo /usr/bin/tor --defaults-torrc tor-service-defaults-torrc -f torrc --RunAsDaemon 0`
2. Once you have run `tor` for the first time get the onion address
   from the `hostname` file (if you used the `tor` invocation above
   this will be in `/var/lib/tor/hidden_service/hostname`). Set the
   onion address `const ONION` in `main.rs`.
3. Set the log level in `main.rs` if you wish.

Now to run this demo application run tor in one terminal, the listener
in another terminal, and the dialer in a third terminal.

- Run the listener with: `ping-pong --listener`.
- Run the dialer with: `ping-pong --dialer`.

See `ping-pong --help` for more information.


Version 0.2 no longer uses the Tor Control Protocol or the `torut`
library to access it.

## ping-pong v0.1

    git checkout v0.1

For the listener the upstream TCP Transport logic is used (albeit
imported into this repository). We use the
[torut](https://github.com/teawithsand/torut) library to start and
connect to a local Tor instance using the [Tor Control
Protocol](https://gitweb.torproject.org/torspec.git/tree/control-spec.txt)
(hats off to the authors, nice library).

For the dialer we modify the TCP Transport's `dial` method to first
connect to the Tor instance started by the listener [2] via a socks5
proxy connection. We convert the Multiaddr (onion) to a format that
Tor can understand and then connect to the onion service using TCP via
the socks5 proxy. For the socks5 connection we use the
[tokio-socks](https://github.com/sticnarf/tokio-socks) library.

The code base is tied to `tokio` as both the `torut` and `tokio-socks`
libraries use `tokio`.

[1] Uses a simple ping application based on the example code from
[rust-libp2p](https://github.com/libp2p/rust-libp2p/blob/master/examples/ping.rs)

[2] Tested on a single machine only.


## Usage

0. Install Tor
1. Make sure Tor can be started but is _not_ currently running (see
   below for the Tor command that is used)
2. In one terminal, run the listener (as root, see below for reason)
3. In another terminal, run the dialer (does not need root)

See `ping-pong --help` for help.

The listener:

```
sudo target/debug/ping-pong
2020-06-25 17:19:01,915 INFO  [ping_pong] Tor instance started

Onion service available at:

    /onion3/jymc37wy2zeiv3y42e2wd6aaqmhl6ckgn4lgdapdw2oln4ydkorqaaad:7777


Local ping server available at:

    /ip4/127.0.0.1/tcp/7777

PingEvent { peer: PeerId("12D3KooWKWFYYkyEhaEFLeMerJ1AEYjAQYfhis1YuZjA1HWwkSxh"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWKWFYYkyEhaEFLeMerJ1AEYjAQYfhis1YuZjA1HWwkSxh"), result: Ok(Ping { rtt: 1.684465016s }) }
PingEvent { peer: PeerId("12D3KooWKWFYYkyEhaEFLeMerJ1AEYjAQYfhis1YuZjA1HWwkSxh"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWKWFYYkyEhaEFLeMerJ1AEYjAQYfhis1YuZjA1HWwkSxh"), result: Ok(Pong) }
```

The dialer:

```
target/debug/ping-pong --dialer --onion /onion3/jymc37wy2zeiv3y42e2wd6aaqmhl6ckgn4lgdapdw2oln4ydkorqaaad:7777
[sudo] password for tobin:
2020-06-25 17:19:39,999 INFO  [ping_pong::transport] connecting to Tor proxy ...
2020-06-25 17:19:49,375 INFO  [ping_pong::transport] connection established
PingEvent { peer: PeerId("12D3KooWPuUb6JCbux9dvKKBweFWbejf3A6fZgntmV26fR9HHqPt"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWPuUb6JCbux9dvKKBweFWbejf3A6fZgntmV26fR9HHqPt"), result: Ok(Ping { rtt: 1.783064395s }) }
PingEvent { peer: PeerId("12D3KooWPuUb6JCbux9dvKKBweFWbejf3A6fZgntmV26fR9HHqPt"), result: Ok(Ping { rtt: 1.866048209s }) }

```


### Further usage notes

Tested with:
```
Tor version 0.4.3.5.
```

The listener must be run with root privileges because of how we start Tor
```
[warn] Bind to /run/tor/socks failed: Permission denied.
```

The command we use to start Tor is
```
/usr/bin/tor --defaults-torrc /usr/share/tor/tor-service-defaults-torrc -f /etc/tor/torrc
```

Copies of both files can be found in this repository.


Thanks for looking,
happy hacking!