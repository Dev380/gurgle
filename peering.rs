use libp2p::{
    gossipsub::{Gossipsub, GossipsubConfig, MessageAuthenticity, Topic},
    identity::Keypair,
    tcp::TcpConfig,
    PeerId, Swarm,
};

struct NodeNetwork {}

pub fn bootstrap() {
    let k = Keypair::generate_ed25519();
    let id = PeerId::from_public_key(k.public());
    let tcp = TokioTcpConfig::new();
    let yamux = yamux::YamuxConfig::default();
    let noise = noise::NoiseConfig::xx(noise_keys).into_authenticated();
    let transport = tcp
        .upgrade(upgrade::Version::V1)
        .authenticate(noise)
        .multiplex(yamux)
        .boxed();
    let swarm = {
        let config = GossipsubConfig {
            ..Default::default()
        };
        let gsub = Gossipsub::new(MessageAuthenticity::Signed(k), config);
        gsub.subscribe(Topic::new("Cowchain".to_string()));
        SwarmBuilder::new(transport, gsub, id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };
}
