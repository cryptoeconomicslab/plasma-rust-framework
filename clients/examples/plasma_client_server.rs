use abi_utils::{Decodable, Integer};
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use chrono::{DateTime, Local};
use env_logger;
use ethereum_types::Address;
use log::info;
use ovm::types::StateUpdate;
use plasma_clients::plasma::{
    error::{Error, ErrorKind},
    query::query_exchanged,
    utils::*,
    PlasmaClientShell,
};
use plasma_core::data_structure::{Range, EXCHANGE_TYPE, PAYMENT_TYPE};
use serde::{Deserialize, Serialize};

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

fn get_all_tokens(plasma_client: web::Data<PlasmaClientShell>) -> Result<HttpResponse> {
    let tokens = plasma_client.get_all_tokens();
    Ok(HttpResponse::Ok().json(tokens))
}

// Get Balance
#[derive(Deserialize, Debug)]
struct GetBalanceRequest {
    session: String,
}

#[derive(Serialize)]
struct Balance {
    token_address: Address,
    token_name: String,
    balance: u64,
}

fn get_balance(
    params: web::Query<GetBalanceRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    info!("PARAMS: {:?}", params);
    let session = decode_session(params.session.clone()).unwrap();
    let balance: Vec<Balance> = plasma_client
        .get_balance(&session)
        .iter()
        .map(|(k, v)| Balance {
            token_address: *k,
            token_name: plasma_client.get_token_name(*k),
            balance: *v,
        })
        .collect();

    Ok(HttpResponse::Ok().json(balance))
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
    token_name: String,
}

fn get_payment_history(
    params: web::Query<GetPaymentHistoryRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    info!("PARAMS: {:?}", params);
    let session = decode_session(params.session.clone()).unwrap();
    let my_address = plasma_client.get_my_address(&session).unwrap();
    let txs = plasma_client.get_related_transactions(&session);
    let history: Vec<PaymentHistory> = txs
        .into_iter()
        .filter_map(|tx| {
            let metadata = tx.get_metadata();
            if metadata.get_meta_type() == PAYMENT_TYPE {
                let send = metadata.get_from() == my_address;
                Some(PaymentHistory {
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
                    token_name: plasma_client.get_token_name(tx.get_deposit_contract_address()),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(history))
}

// Send Payment
#[derive(Deserialize, Serialize, Debug)]
struct SendPayment {
    token_address: Address,
    from: Address,
    to: Address,
    amount: u64,
    session: String,
}

fn send_payment(
    body: web::Json<SendPayment>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    let session = decode_session(body.session.clone()).unwrap();
    let account = plasma_client.get_my_address(&session).unwrap();
    if let Some(range) = plasma_client.search_range(body.token_address, body.amount, account) {
        println!("Range: {:?}", range);
        let (property, metadata) = plasma_client.ownership_property(&session, body.to);
        plasma_client.send_transaction(
            &session,
            Some(body.token_address),
            range.get_start(),
            range.get_start() + body.amount,
            property,
            metadata,
        );
        return Ok(HttpResponse::Ok().json(SendPayment {
            token_address: body.token_address,
            from: body.from,
            to: body.to,
            session: body.session.clone(),
            amount: body.amount,
        }));
    }

    Err(error::ErrorBadRequest(Error::from(
        ErrorKind::InvalidParameter,
    )))
}

// Get Exchange Offers
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct CounterParty {
    token_address: Address,
    amount: u64,
    address: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExchangeOffer {
    exchange_id: String,
    token_address: Address,
    start: u64,
    end: u64,
    counter_party: CounterParty,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExchangeOfferResponse {
    exchange_id: String,
    token_address: Address,
    amount: u64,
    counter_party: CounterParty,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExchangeOfferRequest {
    token_address: Address,
    amount: u64,
    counter_party: CounterParty,
}

fn get_exchange_offers(plasma_client: web::Data<PlasmaClientShell>) -> Result<HttpResponse> {
    let orders: Vec<ExchangeOfferResponse> = plasma_client
        .get_orders()
        .iter()
        .map(
            |(state_update, token_address, amount, maker)| ExchangeOfferResponse {
                // TODO: get exchange_id
                exchange_id: encode_hex(&state_update.get_hash()),
                token_address: state_update.get_deposit_contract_address(),
                amount: state_update.get_range().get_end() - state_update.get_range().get_start(),
                counter_party: CounterParty {
                    token_address: *token_address,
                    amount: amount.0,
                    address: Some(*maker),
                },
            },
        )
        .collect();
    Ok(HttpResponse::Ok().json(orders))
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
    session: String,
}

#[derive(Serialize)]
struct ExchangeHistory {
    exchange_id: String,
    history_type: ExchangeHistoryType,
    token_address: Address,
    amount: u64,
    status: ExchangeHistoryStatus,
    counter_party: CounterParty,
    timestamp: DateTime<Local>,
}

fn get_exchange_history(
    params: web::Query<GetExchangeHistoryRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    let session = decode_session(params.session.clone()).unwrap();
    let my_address = plasma_client.get_my_address(&session).unwrap();
    let txs = plasma_client.get_related_transactions(&session);
    let history: Vec<ExchangeHistory> = txs
        .into_iter()
        .filter_map(|tx| {
            let metadata = tx.get_metadata();
            let state_update = StateUpdate::from_abi(tx.get_parameters()).unwrap();
            if metadata.get_meta_type() == EXCHANGE_TYPE {
                let (c_token, c_range) = query_exchanged(state_update).unwrap();
                let send = metadata.get_from() == my_address;
                Some(ExchangeHistory {
                    exchange_id: "00".to_string(),
                    history_type: if send {
                        ExchangeHistoryType::OFFERED
                    } else {
                        ExchangeHistoryType::OFFER
                    },
                    token_address: tx.get_deposit_contract_address(),
                    amount: tx.get_range().get_amount(),
                    status: ExchangeHistoryStatus::CONFIRMED,
                    counter_party: CounterParty {
                        token_address: c_token,
                        amount: c_range.get_amount(),
                        address: None,
                    },
                    timestamp: Local::now(),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(HttpResponse::Ok().json(history))
}

// Send Exchange
#[derive(Serialize, Deserialize, Debug)]
struct SendExchange {
    from: Address,
    exchange_id: String,
    session: String,
}

fn send_exchange(
    body: web::Json<SendExchange>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    let session = decode_session(body.session.clone()).unwrap();
    let account = plasma_client.get_my_address(&session).unwrap();
    let orders: Vec<ExchangeOffer> = plasma_client
        .get_orders()
        .iter()
        .map(
            |(state_update, token_address, amount, maker)| ExchangeOffer {
                // TODO: get exchange_id
                exchange_id: encode_hex(&state_update.get_hash()),
                token_address: state_update.get_deposit_contract_address(),
                start: state_update.get_range().get_start(),
                end: state_update.get_range().get_end(),
                counter_party: CounterParty {
                    token_address: *token_address,
                    amount: amount.0,
                    address: Some(*maker),
                },
            },
        )
        .filter(|offer| body.exchange_id.clone() == offer.exchange_id)
        .collect();
    if let Some(order) = orders.first() {
        if let Some(range) = plasma_client.search_range(
            order.counter_party.token_address,
            order.counter_party.amount,
            account,
        ) {
            let will_update_range = Range::new(
                range.get_start(),
                range.get_start() + order.counter_party.amount,
            );
            let maker = order.counter_party.address.unwrap();
            let (property1, metadata1) = plasma_client.ownership_property(&session, maker);
            let (property2, metadata2) = plasma_client.taking_order_property(
                &session,
                maker,
                order.counter_party.token_address,
                will_update_range,
            );
            plasma_client.send_transaction(
                &session,
                Some(order.counter_party.token_address),
                will_update_range.get_start(),
                will_update_range.get_end(),
                property1,
                metadata1,
            );
            plasma_client.send_transaction(
                &session,
                Some(order.token_address),
                order.start,
                order.end,
                property2,
                metadata2,
            );
            Ok(HttpResponse::Ok().json(SendExchange {
                from: body.from,
                exchange_id: body.exchange_id.clone(),
                session: body.session.clone(),
            }))
        } else {
            Err(error::ErrorBadRequest(Error::from(
                ErrorKind::InvalidParameter,
            )))
        }
    } else {
        Err(error::ErrorBadRequest(Error::from(
            ErrorKind::InvalidParameter,
        )))
    }
}

// Create Exchange Offer
#[derive(Deserialize, Serialize, Debug)]
struct CreateExchangeOfferRequest {
    from: Address,
    offer: ExchangeOfferRequest,
    session: String,
}

fn create_exchange_offer(
    body: web::Json<CreateExchangeOfferRequest>,
    plasma_client: web::Data<PlasmaClientShell>,
) -> Result<HttpResponse> {
    let session = decode_session(body.session.clone()).unwrap();
    let account = plasma_client.get_my_address(&session).unwrap();
    if let Some(range) =
        plasma_client.search_range(body.offer.token_address, body.offer.amount, account)
    {
        let (property, metadata) = plasma_client.making_order_property(
            &session,
            body.offer.counter_party.token_address,
            Integer(body.offer.counter_party.amount),
        );
        plasma_client.send_transaction(
            &session,
            Some(body.offer.token_address),
            range.get_start(),
            range.get_start() + body.offer.amount,
            property,
            metadata,
        );
        Ok(HttpResponse::Ok().json(CreateExchangeOfferRequest {
            from: body.from,
            offer: body.offer.clone(),
            session: body.session.clone(),
        }))
    } else {
        Err(error::ErrorBadRequest(Error::from(
            ErrorKind::InvalidParameter,
        )))
    }
}

pub fn main() {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    HttpServer::new(move || {
        let mut client = PlasmaClientShell::new(
            "client", // db name
            "127.0.0.1:8080".to_owned(),
            string_to_address("9FBDa871d559710256a2502A2517b794B482Db40"),
        );
        client.connect();
        client.initialize();
        let data = web::Data::new(client);
        App::new()
            .wrap(Logger::default())
            .register_data(data)
            .route("/create_account", web::post().to(create_account))
            .route("/get_all_tokens", web::post().to(get_all_tokens))
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
    .bind("0.0.0.0:7777")
    .unwrap()
    .run()
    .unwrap();
}
