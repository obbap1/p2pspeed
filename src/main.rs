mod messaging; 
mod services;

use futures::prelude::*;
use libp2p::core::upgrade::Version;
use libp2p_swarm_derive::NetworkBehaviour;
use libp2p::{
    identity, noise, gossipsub, mdns,ping,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport
};


use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::Hasher;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::async_io::Behaviour
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local Peer ID: {local_peer_id}");
    
    let transport = tcp::async_io::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        s.write(&message.data);
        let message_id = s.finish().to_string();
        println!("Message ID is {message_id}");
        gossipsub::MessageId::from(message_id)
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(30))
        .message_id_fn(message_id_fn)
        .build()
        .expect("Valid gossipsub config");

    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key), 
        gossipsub_config
    ).expect("Correct configuration");

    let topic = gossipsub::IdentTopic::new("internet-speed");

    gossipsub.subscribe(&topic).expect("Subscribed to topic successfully");

    let mut swarm = {
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id).expect("mDNS config created successfully");
        let behaviour = MyBehaviour {gossipsub, mdns};
        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id)
        .build()
    };

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // dial bootstrap node
    if let Some(addr) = std::env::args().nth(1){
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { listener_id, address } => println!("Listener ID {listener_id:?} and Address {address}"),
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                 for (peer_id, multiaddr) in list {
                    println!("Discovered peer with ID: {peer_id} and address {multiaddr}");
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id)
                 }
            },
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for(peer_id, multiaddr) in list {
                    println!("Expired peer with ID: {peer_id} and address {multiaddr}");
                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id)
                }

            },
            SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message { propagation_source, message_id, message })) => {
                println!("Got message '{}' from {propagation_source} and ID: {message_id}",
                String::from_utf8(message.data).expect("utf8 string"))
            },
            _ => {},
        }
    }

}
