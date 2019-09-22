use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use bytes::Bytes;
use chrono::{DateTime, Local};
use env_logger;
use ethereum_types::Address;
use log::info;
use plasma_clients::plasma::{
    error::{Error, ErrorKind},
    PlasmaClientShell,
};
use serde::{Deserialize, Serialize};

fn decode_session(session: String) -> Result<Bytes, ()> {
    if let Ok(s) = hex::decode(session) {
        Ok(Bytes::from(s))
    } else {
        Err(())
    }
}

fn encode_session(raw: Bytes) -> String {
    hex::encode(raw.to_vec())
}

// Create Account
#[derive(Serialize)]
struct CreateAccountResponse {
    address: Address,
    session: String,
}

fn create_account(plasma_client: web::Data<PlasmaClientShell>) -> Result<HttpResponse> {
    let (session, key) = plasma_client.create_account();
    Ok(HttpResponse::Ok().json(CreateAccountResponse {
        address: Address::from(key.public().address()),
        session: encode_session(session),
    }))
}

// Get Balance
#[derive(Deserialize, Debug)]
struct GetBalanceRequest {
    session: String,
}

#[derive(Serialize)]
struct Balance {
    token_id: u64,
    balance: u64,
}

fn get_balance(
    body: web::Json<GetBalanceRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    let session = decode_session(body.session.clone()).unwrap();
    let balance = plasma_client.get_balance(&session);
    Ok(HttpResponse::Ok().json(vec![Balance {
        token_id: 1,
        balance,
    }]))
}

// Get Payment History
#[derive(Deserialize, Serialize)]
enum PaymentHistoryType {
    SEND,
    RECEIVE,
}

#[derive(Deserialize, Serialize)]
enum PaymentHistoryStatus {
    CONFIRMED,
    PENDING,
    FAILED,
}

#[derive(Deserialize, Debug)]
struct GetPaymentHistoryRequest {
    session: String,
}

#[derive(Serialize)]
struct PaymentHistory {
    history_type: PaymentHistoryType,
    amount: u64,
    address: Address,
    timestamp: DateTime<Local>,
    status: PaymentHistoryStatus,
}

fn get_payment_history(
    body: web::Json<GetPaymentHistoryRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    let session = decode_session(body.session.clone()).unwrap();
    let my_address = plasma_client.get_my_address(&session).unwrap();
    let txs = plasma_client.get_related_transactions(&session);
    println!("{:?}", txs);
    let history: Vec<PaymentHistory> = txs
        .into_iter()
        .map(|tx| {
            let metadata = tx.get_metadata();
            let send = metadata.get_from() == my_address;
            PaymentHistory {
                history_type: if send {
                    PaymentHistoryType::SEND
                } else {
                    PaymentHistoryType::RECEIVE
                },
                amount: tx.get_range().get_amount(),
                address: if send {
                    metadata.get_to()
                } else {
                    metadata.get_from()
                },
                timestamp: Local::now(),
                status: PaymentHistoryStatus::CONFIRMED,
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(history))
}

// Send Payment
#[derive(Deserialize, Serialize, Debug)]
struct SendPayment {
    from: Address,
    to: Address,
    amount: u64,
    token_id: u64,
    session: String,
}

fn send_payment(
    body: web::Json<SendPayment>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    if let Some(range) = plasma_client.search_range(body.amount) {
        let session = decode_session(body.session.clone()).unwrap();
        plasma_client.send_transaction(
            &session,
            body.to,
            range.get_start(),
            range.get_start() + body.amount,
        );

        return Ok(HttpResponse::Ok().json(SendPayment {
            from: body.from,
            to: body.to,
            session: body.session.clone(),
            amount: body.amount,
            token_id: body.token_id,
        }));
    }

    Err(error::ErrorBadRequest(Error::from(
        ErrorKind::InvalidParameter,
    )))
}

// Get Exchange Offers
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct CounterParty {
    token_id: u64,
    amount: u64,
    address: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct ExchangeOffer {
    exchange_id: u64,
    token_id: u64,
    amount: u64,
    counter_party: CounterParty,
}

fn get_exchange_offers() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(vec![
        ExchangeOffer {
            exchange_id: 1234,
            token_id: 1,
            amount: 10,
            counter_party: CounterParty {
                token_id: 2,
                amount: 1,
                address: None,
            },
        },
        ExchangeOffer {
            exchange_id: 123,
            token_id: 1,
            amount: 10,
            counter_party: CounterParty {
                token_id: 2,
                amount: 1,
                address: Some(Address::zero()),
            },
        },
    ]))
}

// Get Exchange History
#[derive(Deserialize, Serialize)]
enum ExchangeHistoryType {
    OFFER,
    OFFERED,
}

#[derive(Deserialize, Serialize)]
enum ExchangeHistoryStatus {
    CONFIRMED,
    PENDING,
    FAILED,
}

#[derive(Deserialize, Debug)]
struct GetExchangeHistoryRequest {
    address: Address,
}

#[derive(Serialize)]
struct ExchangeHistory {
    exchange_id: u64,
    history_type: ExchangeHistoryType,
    token_id: u64,
    amount: u64,
    status: ExchangeHistoryStatus,
    counter_party: CounterParty,
    timestamp: DateTime<Local>,
}

fn get_exchange_history(body: web::Json<GetExchangeHistoryRequest>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(vec![ExchangeHistory {
        exchange_id: 123,
        history_type: ExchangeHistoryType::OFFERED,
        token_id: 1,
        amount: 10,
        status: ExchangeHistoryStatus::CONFIRMED,
        counter_party: CounterParty {
            token_id: 2,
            amount: 1,
            address: Some(Address::zero()),
        },
        timestamp: Local::now(),
    }]))
}

// Send Exchange
#[derive(Serialize, Deserialize, Debug)]
struct SendExchange {
    from: Address,
    exchange_id: u64,
}

fn send_exchange(body: web::Json<SendExchange>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(SendExchange {
        from: body.from,
        exchange_id: body.exchange_id,
    }))
}

// Create Exchange Offer
#[derive(Deserialize, Serialize, Debug)]
struct CreateExchangeOfferRequest {
    from: Address,
    offer: ExchangeOffer,
}

fn create_exchange_offer(body: web::Json<CreateExchangeOfferRequest>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(CreateExchangeOfferRequest {
        from: body.from,
        offer: body.offer,
    }))
}

pub fn main() {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    let commitment_contract_address_hex =
        hex::decode("9FBDa871d559710256a2502A2517b794B482Db40").unwrap();
    let commitment_contract_address = Address::from_slice(&commitment_contract_address_hex);

    HttpServer::new(move || {
        let mut client =
            PlasmaClientShell::new("127.0.0.1:8080".to_owned(), commitment_contract_address);
        client.connect();

        let data = web::Data::new(client);
        App::new()
            .wrap(Logger::default())
            .register_data(data)
            .route("/create_account", web::post().to(create_account))
            .route("/get_balance", web::get().to(get_balance))
            .route("/get_payment_history", web::get().to(get_payment_history))
            .route("/send_payment", web::post().to(send_payment))
            .route("/get_exchange_offers", web::get().to(get_exchange_offers))
            .route("/get_exchange_history", web::get().to(get_exchange_history))
            .route("/send_exchange", web::post().to(send_exchange))
            .route(
                "/create_exchange_offer",
                web::post().to(create_exchange_offer),
            )
    })
    .workers(1)
    .bind("127.0.0.1:7777")
    .unwrap()
    .run()
    .unwrap();
}
