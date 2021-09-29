mod account;
mod block;
mod chain;
mod node;
mod p2p;
mod transaction;
mod transaction_pool;
mod wallet;




use libp2p::{
  core::upgrade,
  floodsub::{self, Floodsub, FloodsubEvent},
  identity,
  mdns::{Mdns, MdnsEvent},
  mplex, noise,
  swarm::{NetworkBehaviourEventProcess, SwarmBuilder, SwarmEvent},
  tcp::TokioTcpConfig,
  Multiaddr, NetworkBehaviour, PeerId, Transport,
};

use tokio::io::{self, AsyncBufReadExt};

use futures::StreamExt;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let peer_id_keys = identity::Keypair::generate_ed25519();
  let peer_id = PeerId::from(peer_id_keys.public());

  println!("peer id: {:?}", &peer_id);

  let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
    .into_authentic(&peer_id_keys)
    .expect("couldn't sign libp2p-noise static DH keypair");

  let transport = TokioTcpConfig::new()
    .nodelay(true)
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
    .multiplex(mplex::MplexConfig::new())
    .boxed();

  let floodsub_topic = floodsub::Topic::new("chat");

  #[derive(NetworkBehaviour)]
  #[behaviour(event_process = true)]
  struct MyBheaviour {
    floodsub: Floodsub,
    mdns: Mdns,
  }

  impl NetworkBehaviourEventProcess<FloodsubEvent> for MyBheaviour {
    fn inject_event(&mut self, message: FloodsubEvent) {
      if let FloodsubEvent::Message(message) = message {
        println!(
          "received {:?} from {:?}",
          String::from_utf8_lossy(&message.data),
          message.source,
        );
      }
    }
  }

  impl NetworkBehaviourEventProcess<MdnsEvent> for MyBheaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
      match event {
        MdnsEvent::Discovered(list) => {
          for (peer, _) in list {
            self.floodsub.add_node_to_partial_view(peer)
          }
        }
        MdnsEvent::Expired(list) => {
          for (peer, _) in list {
            if !self.mdns.has_node(&peer) {
              self.floodsub.remove_node_from_partial_view(&peer);
            }
          }
        }
      }
    }
  }

  let mut swarm = {
    let mdns = Mdns::new(Default::default()).await?;

    let mut behaviour = MyBheaviour {
      floodsub: Floodsub::new(peer_id.clone()),
      mdns,
    };

    behaviour.floodsub.subscribe(floodsub_topic.clone());

    SwarmBuilder::new(transport, behaviour, peer_id)
      .executor(Box::new(|fut| {
        tokio::spawn(fut);
      }))
      .build()
  };

  if let Some(to_dial) = std::env::args().nth(1) {
    let address: Multiaddr = to_dial.parse()?;

    swarm.dial_addr(address)?;

    println!("dialed {:?}", to_dial);
  }

  let mut stdin = io::BufReader::new(io::stdin()).lines();

  swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

  loop {
    tokio::select! {
      line = stdin.next_line() => {
        let line = line?.expect("stdin closed");
        swarm.behaviour_mut().floodsub.publish(floodsub_topic.clone(), line.as_bytes());
      }
      event = swarm.select_next_some() => {
        if let SwarmEvent::NewListenAddr{address,..} = event {
          println!("listening on {:?}",address);
        }
      }
    }
  }
}
