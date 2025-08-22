# WebRTC PoC

## Motivation

After trying to support WebRTC transport in `lexnv/litep2p-perf` ([draft PR](https://github.com/timwu20/litep2p-perf/pull/1)), I tried to start from scratch only supporting WebRTC transport by utilizing `libp2p-webrtc@v0.9.0-alpha`.  Started first by using `libp2p-ping` behaviour and verified that it worked fine.  Next moved on to importing `libp2p-perf` which is a performance testing behaviour that gets the client to upload a number of bytes, download a number bytes, and report the speed.  Got it to work up byte counts of 7KB.  Anything more generates the following error on the server side:

```
Custom { kind: InvalidData, error: Error(Custom { kind: Other, error: "Short buffer (size: 8192) to be filled" }) }
```

This was then traced to [this line](https://github.com/libp2p/rust-asynchronous-codec/blob/master/src/framed_read.rs#L186) in `asynchronous-codec` crate.  It looks to be that the max length of `buf` is 8192, so it looks like the message is overflowing the buffer which then returns this error.  

I then tried to add `yamux` to the WebRTC `Transport`, but never could figure out how to add it.  Getting lots of unfulfilled trait errors when trying to do it based on example code.

## How to Run

From root directory run each binary independently.
```
cargo run --package webrtc-poc-server --bin webrtc-poc-server
cargo run --package webrtc-poc-client --bin webrtc-poc-client
```