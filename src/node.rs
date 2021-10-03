use crate::chain::Chain;
use crate::transaction::PublicKey;
use crate::wallet::{SignedTransaction, Wallet};
use std::collections::HashSet;

use libp2p::{
  core::upgrade,
  floodsub::{self, Floodsub, FloodsubEvent},
  identity,
  mdns::{Mdns, MdnsEvent},
  mplex, noise,
  swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder, SwarmEvent},
  tcp::TokioTcpConfig,
  NetworkBehaviour, PeerId, Transport,
};

use tokio::{
  io::{self, AsyncBufReadExt},
  runtime::Runtime,
};

use std::pin::Pin;

use std::sync::{
  mpsc,
  mpsc::{Receiver, Sender},
  Arc, Mutex,
};

use futures::{Future, StreamExt};

use std::error::Error;

pub struct Node {
  transactions: HashSet<SignedTransaction>,
  wallet: Wallet,
  chain: Chain,
}

#[derive(Debug, PartialEq)]
pub enum NodeError {
  InvalidSignature {
    public_key: PublicKey,
    signed_transaction: SignedTransaction,
  },
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct NodeBehaviour {
  floodsub: Floodsub,
  mdns: Mdns,
}

lazy_static! {
  static ref CHANNEL: (Mutex<Sender<String>>, Mutex<Receiver<String>>) = {
    let (sender, receiver) = mpsc::channel();
    (Mutex::new(sender), Mutex::new(receiver))
  };
  static ref FLOODSUB_TOPIC: floodsub::Topic = floodsub::Topic::new("network");
  static ref SWARM: Arc<Mutex<Swarm<NodeBehaviour>>> = {
    Runtime::new().unwrap().block_on(async {
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

      impl NetworkBehaviourEventProcess<FloodsubEvent> for NodeBehaviour {
        fn inject_event(&mut self, message: FloodsubEvent) {
          if let FloodsubEvent::Message(message) = message {
            let payload = format!(
              "received {:?} from {:?}",
              String::from_utf8_lossy(&message.data),
              message.source,
            );

            CHANNEL.0.lock().unwrap().send(payload.clone()).unwrap();

            println!("{}", payload);
          }
        }
      }

      impl NetworkBehaviourEventProcess<MdnsEvent> for NodeBehaviour {
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
        let mdns = Mdns::new(Default::default()).await.unwrap();
        let mut behaviour = NodeBehaviour {
          floodsub: Floodsub::new(peer_id.clone()),
          mdns,
        };

        behaviour.floodsub.subscribe(FLOODSUB_TOPIC.clone());
        SwarmBuilder::new(transport, behaviour, peer_id)
          .executor(Box::new(|fut| {
            tokio::spawn(fut);
          }))
          .build()
      };

      swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();

      Arc::new(Mutex::new(swarm))
    })
  };
}

impl Node {
  async fn handle_swarm_messages() -> impl Future<Output = ()> + Send {
    let swarm_clone = Arc::clone(&SWARM);
    async move {
      let mut stdin = io::BufReader::new(io::stdin()).lines();
      loop {
        let swarm = swarm_clone.lock().unwrap();
        tokio::select! {
          line = stdin.next_line() => {
            let line = line.unwrap().expect("stdin closed");
            swarm.behaviour_mut().floodsub.publish(FLOODSUB_TOPIC.clone(), line.as_bytes());
          }
          event = swarm.select_next_some() => {
            if let SwarmEvent::NewListenAddr { address, .. } = event {
                println!("Listening on {:?}", address);
            }
          }
        }
        drop(swarm);
      }
    }
  }

  pub async fn new() -> Result<Self, Box<dyn Error>> {
    // let s: dyn Future<Output = ()> + Send = async move {
    //   let mut stdin = io::BufReader::new(io::stdin()).lines();
    //   loop {
    //     let swarm = swarm_clone.lock().unwrap();
    //     tokio::select! {
    //       line = stdin.next_line() => {
    //         let line = line.unwrap().expect("stdin closed");
    //         swarm.behaviour_mut().floodsub.publish(FLOODSUB_TOPIC.clone(), line.as_bytes());
    //       }
    //       event = swarm.select_next_some() => {
    //         if let SwarmEvent::NewListenAddr { address, .. } = event {
    //             println!("Listening on {:?}", address);
    //         }
    //       }
    //     }
    //     drop(swarm);
    //   }
    // };
    tokio::task::spawn(Node::handle_swarm_messages);

    Ok(Self {
      transactions: HashSet::new(),
      wallet: Wallet::new(),
      chain: Chain::new(),
    })
  }

  pub fn transaction(
    &mut self,
    public_key: &PublicKey,
    transaction: SignedTransaction,
  ) -> Result<(), NodeError> {
    if false && !Wallet::verify_transaction(public_key, &transaction) {
      return Err(NodeError::InvalidSignature {
        public_key: public_key.clone(),
        signed_transaction: transaction.clone(),
      });
    }

    let transaction_wasnt_in_the_set = self.transactions.insert(transaction);

    if transaction_wasnt_in_the_set {
      // self
      //   .sender
      //   .lock()
      //   .unwrap()
      //   .send(String::from("new transaction added homie"))
      //   .unwrap();
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::transaction::Transaction;

  #[actix_rt::test]
  async fn returns_error_when_we_try_to_add_a_transaction_with_an_invalid_signature() {
    let wallet_a = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let transaction_signed_by_wallet_a = wallet_a.sign_transaction(transaction.clone());

    let wallet_b = Wallet::new();

    let mut node = Node::new().await.unwrap();

    let expected = Err(NodeError::InvalidSignature {
      public_key: wallet_b.public_key(),
      signed_transaction: transaction_signed_by_wallet_a.clone(),
    });

    let actual = node.transaction(&wallet_b.public_key(), transaction_signed_by_wallet_a);

    assert_eq!(expected, actual);
  }

  #[actix_rt::test]
  async fn adds_transaction_to_transaction_set() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign_transaction(transaction.clone());

    let mut node = Node::new().await.unwrap();

    let mut expected = HashSet::new();

    expected.insert(signed_transaction.clone());

    let actual = node.transaction(&wallet.public_key(), signed_transaction);

    assert_eq!(Ok(()), actual);

    assert_eq!(node.transactions, expected);
  }

  #[actix_rt::test]
  async fn each_transaction_is_only_added_once() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign_transaction(transaction.clone());

    let mut node = Node::new().await.unwrap();

    let mut expected = HashSet::new();

    expected.insert(signed_transaction.clone());

    node
      .transaction(&wallet.public_key(), signed_transaction.clone())
      .unwrap();
    node
      .transaction(&wallet.public_key(), signed_transaction)
      .unwrap();

    assert_eq!(node.transactions, expected);
  }
}
