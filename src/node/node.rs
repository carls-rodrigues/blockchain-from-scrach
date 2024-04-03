use std::collections::HashMap;

use crate::database::{new_account, State, Tx};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use data_encoding::HEXLOWER;

const HTTP_PORT: u16 = 8080;

#[derive(Debug, serde::Serialize)]
struct BalanceRes {
    balances: HashMap<String, u64>,
    #[serde(rename = "block_hash")]
    hash: String,
}

#[derive(Debug, serde::Deserialize)]
struct TxAddReq {
    from: String,
    to: String,
    value: u64,
    data: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct TxAddRes {
    #[serde(rename = "block_hash")]
    hash: String,
}

#[actix_web::get("/balances/list")]
async fn list_balances_handler(data_dir: web::Data<String>) -> impl Responder {
    let state = State::new_state_from_disk(&data_dir);
    state.close();
    let output = BalanceRes {
        balances: state.get_balances().clone(),
        hash: HEXLOWER.encode(&state.latest_block_hash()),
    };
    HttpResponse::Ok().json(output)
}

#[actix_web::post("/tx/add")]
async fn tx_add_handler(
    payload: web::Json<TxAddReq>,
    data_dir: web::Data<String>,
) -> impl Responder {
    let mut state = State::new_state_from_disk(&data_dir);
    state.close();
    let from = new_account(&payload.from.clone());
    let to = new_account(&payload.to.clone());
    let value = payload.value;
    let data = match &payload.data {
        Some(data) => data.clone(),
        None => "".to_string(),
    };

    let tx = Tx::new(from, to, &value, &data);
    let Ok(_) = state.add_tx(&tx) else {
        println!("Handler: Error adding transaction");
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(persist_tx) = state.persist() else {
        println!("Handler: Error persisting transaction");
        return HttpResponse::InternalServerError().finish();
    };
    let output = TxAddRes {
        hash: HEXLOWER.encode(&persist_tx),
    };
    HttpResponse::Ok().json(output)
}

#[actix_web::main]
pub async fn run(data_dir: &str) -> std::io::Result<()> {
    println!("Listening on HTTP port: {}", HTTP_PORT);
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let data_dir = data_dir.to_string();
    let _ = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(list_balances_handler)
            .service(tx_add_handler)
            .app_data(web::Data::new(data_dir.clone()))
    })
    .bind(("127.0.0.1", HTTP_PORT))?
    .run()
    .await;
    Ok(())
}
