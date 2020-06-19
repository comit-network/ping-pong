# Ping-Pong

Basic peer to peer connectivity using libp2p. Based on the example
code in [rust-libp2p]()https://github.com/libp2p/rust-libp2p/)
examples/ directory.

## Usage

Run the listener:

```
pingpong
Listening on /ip4/127.0.0.1/tcp/4444
PingEvent { peer: PeerId("12D3KooWKu2mukgUxrbXYBJmSY8j8pqzTWwkmoFzaGtS3ihSAav3"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWKu2mukgUxrbXYBJmSY8j8pqzTWwkmoFzaGtS3ihSAav3"), result: Ok(Ping { rtt: 2.484608ms }) }
PingEvent { peer: PeerId("12D3KooWKu2mukgUxrbXYBJmSY8j8pqzTWwkmoFzaGtS3ihSAav3"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWKu2mukgUxrbXYBJmSY8j8pqzTWwkmoFzaGtS3ihSAav3"), result: Ok(Pong) }
```

In another terminal run the dialer:
```
pingpong --dialer /ip4/127.0.0.1/tcp/4444
PingEvent { peer: PeerId("12D3KooWCwf4Wh4BQARUMbtNHt6RxXCuD78x2eaCLCF2Hx8yQG3d"), result: Ok(Pong) }
PingEvent { peer: PeerId("12D3KooWCwf4Wh4BQARUMbtNHt6RxXCuD78x2eaCLCF2Hx8yQG3d"), result: Ok(Ping { rtt: 2.271353ms }) }
PingEvent { peer: PeerId("12D3KooWCwf4Wh4BQARUMbtNHt6RxXCuD78x2eaCLCF2Hx8yQG3d"), result: Ok(Ping { rtt: 4.227172ms }) }
PingEvent { peer: PeerId("12D3KooWCwf4Wh4BQARUMbtNHt6RxXCuD78x2eaCLCF2Hx8yQG3d"), result: Ok(Ping { rtt: 4.138663ms }) }

```
