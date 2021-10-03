mod account;
mod block;
mod chain;
mod controllers;
mod node;
mod transaction;
mod viewmodel;
mod wallet;

use node::Node;

use std::env;

use actix_web::{web, App, HttpServer};

use std::sync::{Arc, Mutex};

#[macro_use]
extern crate lazy_static;

fn main() {
  actix_web::rt::System::with_tokio_rt(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .worker_threads(8)
      .thread_name("main_tokio")
      .build()
      .unwrap()
  })
  .block_on(async_main());
}

async fn async_main() {
  let node = web::Data::new(Arc::new(Mutex::new(Node::new().await.unwrap())));

  let port = env::args().nth(1).unwrap();

  HttpServer::new(move || {
    App::new()
      .app_data(node.clone())
      .service(controllers::add_transaction)
  })
  .bind(format!("127.0.0.1:{}", port))
  .unwrap()
  .run()
  .await
  .unwrap();
}
