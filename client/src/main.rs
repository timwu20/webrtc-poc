use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, Transport},
    multiaddr::Multiaddr,
    ping,
    swarm::SwarmEvent,
};
use libp2p_webrtc as webrtc;
use rand::thread_rng;
use std::time::Duration;
use libp2p_perf::{client, server, Final, Intermediate, Run, RunParams, RunUpdate};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

     let behav = libp2p_perf::client::Behaviour::new();

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

    let listen_addr = "/ip4/0.0.0.0/udp/0/webrtc-direct".parse()?;
    swarm.listen_on(listen_addr)?;

    let libp2p_endpoint = "/ip4/192.168.68.107/udp/58214/webrtc-direct/certhash/uEiD85hZXNtu7UbexCzSMPzAzpLv2c--R6STG3mV5LUy4Hw/p2p/12D3KooWGcxd5Vr6vkd41Cd4rNErsJE94Wm7sBN2GXCPVxKzqwrr";
    
    tokio::spawn(async move {
        let addr = libp2p_endpoint.parse::<Multiaddr>()?;
        tracing::info!("Dialing {addr}");
        swarm.dial(addr)?;

        
        let server_peer_id = loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => break peer_id,
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    panic!("Failed to dial {libp2p_endpoint}: {error:?} for peer {peer_id:?}");
                }
                SwarmEvent::NewListenAddr { .. } => {
                    continue;
                },
                e => panic!("{e:?}"),
            };
        };

        let params = libp2p_perf::RunParams {
            to_send: 1024 * 8,
            to_receive: 1024 * 8,
        };

        swarm.behaviour_mut().perf(server_peer_id, params)?;

        let duration = loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::Behaviour(client::Event {
                    id: _,
                    result: Ok(RunUpdate::Intermediate(progressed)),
                }) => {
                    tracing::info!("{progressed}");

                    let Intermediate {
                        duration,
                        sent,
                        received,
                    } = progressed;

                    // println!(
                    //     "{}",
                    //     serde_json::to_string(&BenchmarkResult {
                    //         r#type: "intermediate".to_string(),
                    //         time_seconds: duration.as_secs_f64(),
                    //         upload_bytes: sent,
                    //         download_bytes: received,
                    //     })
                    //     .unwrap()
                    // );
                    tracing::info!(
                        "Progress: {}/{} bytes sent, {}/{} bytes received",
                        sent,
                        params.to_send,
                        received,
                        params.to_receive
                    );
                }
                SwarmEvent::Behaviour(client::Event {
                    id: _,
                    result: Ok(RunUpdate::Final(Final { duration })),
                }) => break duration,
                e => panic!("{e:?}"),
            };
        };

        let run = Run { params, duration };

        tracing::info!("{run}");
        
        anyhow::Ok(())
    }).await??;

    Ok(())
}
