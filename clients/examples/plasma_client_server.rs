use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use chrono::{DateTime, Local};
use env_logger;
use ethereum_types::Address;
use log::info;
use serde::{Deserialize, Serialize};

// Create Account
#[derive(Deserialize, Debug)]
struct CreateAccountRequest {
    password_hash: String,
}

#[derive(Serialize)]
struct CreateAccountResponse {
    address: Address,
}

fn create_account(body: web::Json<CreateAccountRequest>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(CreateAccountResponse {
        address: Address::zero(),
    }))
}

// Get Balance
#[derive(Deserialize, Debug)]
struct GetBalanceRequest {
    address: Address,
}

#[derive(Serialize)]
struct GetBalanceResponse {
    balance: u64,
}

fn get_balance(body: web::Json<GetBalanceRequest>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(GetBalanceResponse { balance: 10 }))
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
    address: Address,
}

#[derive(Serialize)]
struct PaymentHistory {
    history_type: PaymentHistoryType,
    amount: u64,
    address: Address,
    timestamp: DateTime<Local>,
    status: PaymentHistoryStatus,
}

fn get_payment_history(body: web::Json<GetPaymentHistoryRequest>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(vec![PaymentHistory {
        history_type: PaymentHistoryType::SEND,
        amount: 10,
        address: Address::zero(),
        timestamp: Local::now(),
        status: PaymentHistoryStatus::CONFIRMED,
    }]))
}

// Send Payment
#[derive(Deserialize, Serialize, Debug)]
struct SendPayment {
    from: Address,
    to: Address,
    amount: u64,
    token_id: u64,
}

fn send_payment(body: web::Json<SendPayment>) -> Result<HttpResponse> {
    info!("BODY: {:?}", body);
    Ok(HttpResponse::Ok().json(SendPayment {
        from: body.from,
        to: body.to,
        amount: body.amount,
        token_id: body.token_id,
    }))
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

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
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
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
