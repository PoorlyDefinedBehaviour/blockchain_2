use crate::node::{Node, NodeError};
use crate::viewmodel;
use crate::wallet::SignedTransaction;
use actix_web::{post, web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};

#[post("/transactions")]
async fn add_transaction(
  node: web::Data<Arc<Mutex<Node>>>,
  input: web::Json<viewmodel::AddTransactionInput>,
) -> impl Responder {
  let mut node = node.lock().unwrap();

  let input = input.into_inner();

  let public_key = input.public_key.clone();

  let input: SignedTransaction = input.into();

  match node.transaction(&public_key, input) {
    Err(NodeError::InvalidSignature { .. }) => {
      HttpResponse::UnprocessableEntity().json(viewmodel::Message {
        message: "invalid signature".to_owned(),
      })
    }
    Ok(()) => HttpResponse::Ok().json(viewmodel::Message {
      message: "transaction added".to_owned(),
    }),
  }
}
