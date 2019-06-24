extern crate hyper;
extern crate jsonrpc_core_client;

use crate::error::Error;
use futures::Future;
use jsonrpc_core_client::transports;
use jsonrpc_core_client::{RpcChannel, RpcError, TypedClient};
use plasma_core::data_structure::{StateQuery, StateQueryResult, Transaction};
use std::cell::RefCell;

#[derive(Clone)]
pub struct RpcClient(TypedClient);

impl From<RpcChannel> for RpcClient {
    fn from(channel: RpcChannel) -> Self {
        RpcClient(channel.into())
    }
}

impl RpcClient {
    pub fn protocol_version(&self) -> impl Future<Item = String, Error = RpcError> {
        self.0.call_method("protocolVersion", "()", ())
    }
    pub fn send_transaction(
        &self,
        transaction: &Transaction,
    ) -> impl Future<Item = bool, Error = RpcError> {
        self.0.call_method(
            "sendTransaction",
            "String",
            (hex::encode(transaction.to_abi()),),
        )
    }
    pub fn send_query(&self, query: &StateQuery) -> impl Future<Item = String, Error = RpcError> {
        self.0
            .call_method("sendQuery", "String", (hex::encode(query.to_abi()),))
    }
}

/// JSON RPC Client over HTTP for Plasma API.
pub struct HttpPlasmaClient {
    runtime: tokio::runtime::Runtime,
    aggregator_client: RefCell<RpcClient>,
}

impl<'a> HttpPlasmaClient {
    pub fn new(uri: &str) -> Result<Self, Error> {
        let mut runtime = tokio::runtime::Runtime::new().map_err::<Error, _>(Into::into)?;
        let aggregator_client: RpcClient = runtime
            .block_on(transports::http::connect::<RpcClient>(uri))
            .map_err::<Error, _>(Into::into)?;
        Ok(Self {
            runtime,
            aggregator_client: RefCell::new(aggregator_client),
        })
    }
    /// Gets protocol version
    pub fn protocol_version(&self) -> Result<String, RpcError> {
        self.aggregator_client
            .borrow_mut()
            .protocol_version()
            .wait()
    }
    /// Sends signed transaction
    pub fn send_transaction(&self, transaction: &Transaction) -> Result<bool, RpcError> {
        self.aggregator_client
            .borrow_mut()
            .send_transaction(transaction)
            .wait()
    }
    /// Sends state query
    pub fn send_query(&self, query: &StateQuery) -> Result<StateQueryResult, Error> {
        self.aggregator_client
            .borrow_mut()
            .send_query(query)
            .wait()
            .map_err::<Error, _>(Into::into)
            .and_then(|result| hex::decode(result).map_err::<Error, _>(Into::into))
            .and_then(|decoded| {
                StateQueryResult::from_abi(&decoded).map_err::<Error, _>(Into::into)
            })
    }
    pub fn shutdown(self) -> std::result::Result<(), ()> {
        self.runtime.shutdown_now().wait()
    }
}

#[cfg(test)]
mod tests {
    extern crate jsonrpc_core_client;

    use super::HttpPlasmaClient;
    use bytes::Bytes;
    use ethereum_types::{Address, H256};
    use jsonrpc_core::{Error, ErrorCode, IoHandler, Params, Value};
    use jsonrpc_http_server::*;
    use plasma_core::data_structure::{
        StateObject, StateQuery, StateQueryResult, StateUpdate, Transaction, Witness,
    };
    use predicate_plugins::{OwnershipPredicateParameters, PredicateParameters};

    #[allow(dead_code)]
    /// from https://github.com/paritytech/jsonrpc/blob/master/core-client/transports/src/transports/http.rs
    struct TestServer {
        uri: String,
        server: Option<Server>,
    }

    impl TestServer {
        fn serve() -> Self {
            let builder = ServerBuilder::new(io()).rest_api(RestApi::Unsecure);

            let server = builder.start_http(&"127.0.0.1:0".parse().unwrap()).unwrap();
            let socket_addr = *server.address();
            let uri = format!("http://{}", socket_addr);

            TestServer {
                uri,
                server: Some(server),
            }
        }
    }

    fn io() -> IoHandler {
        let mut io = IoHandler::default();
        io.add_method("protocolVersion", |_: Params| {
            Ok(Value::String("0.1.0".into()))
        });
        io.add_method("sendTransaction", |_: Params| Ok(Value::Bool(true)));
        io.add_method("sendQuery", |params: Params| {
            match params.parse::<(String,)>() {
                Ok((msg,)) => {
                    let decoded = hex::decode(msg).ok().unwrap();
                    let state_query = StateQuery::from_abi(&decoded).ok().unwrap();
                    let state_update = StateUpdate::new(
                        StateObject::new(Address::zero(), Bytes::from(&b"data"[..])),
                        state_query.get_start().unwrap_or(0),
                        state_query.get_end().unwrap_or(0),
                        10,
                        Address::zero(),
                    );
                    Ok(Value::String(hex::encode(
                        StateQueryResult::new(state_update, &[]).to_abi(),
                    )))
                }
                _ => Err(Error::new(ErrorCode::ServerError(-34))),
            }
        });
        io
    }

    #[test]
    fn test_protocol_version() {
        let server = TestServer::serve();
        let client = HttpPlasmaClient::new(&server.uri).ok().unwrap();
        let r = client.protocol_version();
        assert_eq!("0.1.0", r.ok().unwrap());
        assert!(client.shutdown().is_ok());
    }

    #[test]
    fn test_send_transaction() {
        let parameters = OwnershipPredicateParameters::new(
            StateObject::new(Address::zero(), Bytes::from(Address::zero().as_bytes())),
            5,
            10,
        );
        let transaction = Transaction::new(
            Address::zero(),
            0,
            100,
            Transaction::create_method_id(&b"send(address)"[..]),
            parameters.encode(),
            &Witness::new(H256::zero(), H256::zero(), 0),
        );

        let server = TestServer::serve();
        let client = HttpPlasmaClient::new(&server.uri).ok().unwrap();
        let r = client.send_transaction(&transaction);
        assert_eq!(true, r.ok().unwrap());
        assert!(client.shutdown().is_ok());
    }

    #[test]
    fn test_send_query() {
        let predicate_address = Address::zero();
        let query = StateQuery::new(
            Address::zero(),
            predicate_address,
            Some(0),
            Some(100),
            Bytes::new(),
        );

        let server = TestServer::serve();

        let client = HttpPlasmaClient::new(&server.uri).ok().unwrap();
        let state_query_result = client.send_query(&query).ok().unwrap();

        assert!(state_query_result.get_result().is_empty());
        assert!(client.shutdown().is_ok());
    }

}
