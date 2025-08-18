// #![allow(non_upper_case_globals)]

use std::net::Ipv4Addr;

use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, Transport},
    multiaddr::{Multiaddr, Protocol},
    ping,
    swarm::SwarmEvent,
};
use libp2p_webrtc as webrtc;
use rand::thread_rng;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let behav = libp2p_perf::server::Behaviour::new();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_other_transport(|id_keys| {
            Ok(webrtc::tokio::Transport::new(
                id_keys.clone(),
                webrtc::tokio::Certificate::generate(&mut thread_rng())?,
            )
            .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
        })?
        .with_behaviour(|_| behav)?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    let address_webrtc = Multiaddr::from(Ipv4Addr::UNSPECIFIED)
        .with(Protocol::Udp(0))
        .with(Protocol::WebRTCDirect);

    swarm.listen_on(address_webrtc.clone())?;

    let address = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            if address
                .iter()
                .any(|e| e == Protocol::Ip4(Ipv4Addr::LOCALHOST))
            {
                tracing::debug!(
                    "Ignoring localhost address to make sure the example works in Firefox"
                );
                continue;
            }

            tracing::info!(%address, "Listening");

            break address;
        }
    };

    let addr = address.with(Protocol::P2p(*swarm.local_peer_id()));
    tracing::info!("Peer address: {addr}");

    loop {
        tokio::select! {
            swarm_event = swarm.next() => {
                if let Some(swarm_event) = swarm_event {
                    match swarm_event {
                        // SwarmEvent::Behaviour(ping::Event { result: Err(e), .. }) => {
                        //     tracing::error!("Ping failed: {:?}", e);

                        //     break;
                        // }
                        // SwarmEvent::Behaviour(ping::Event {
                        //     peer,
                        //     result: Ok(rtt),
                        //     ..
                        // }) => {
                        //     tracing::info!("Ping successful: RTT: {rtt:?}, from {peer}");
                        // }
                        SwarmEvent::Behaviour(libp2p_perf::server::Event { remote_peer_id, stats }) => {
                            tracing::info!("Finished run for peer {remote_peer_id}: {stats:?}");
                        }
                        SwarmEvent::ConnectionClosed {
                            cause: Some(cause), ..
                        } => {
                            tracing::info!("Connection closed due to: {:?}", cause);
                        }
                        evt => tracing::info!("Swarm event: {:?}", evt),
                    }
                }
                // tracing::info!(?swarm_event)
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    Ok(())
}
