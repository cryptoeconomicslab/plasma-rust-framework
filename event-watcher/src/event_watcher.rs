use super::event_db::EventDb;
use ethabi::{decode, Error, ErrorKind, Event, EventParam, ParamType, Token, Topic, TopicFilter};
use ethereum_types::{Address, H256};
use futures::{Async, Future, Poll, Stream};
use std::time::Duration;
use tokio::timer::Interval;
use web3::types::{BlockNumber, FilterBuilder, Log as RawLog};
use web3::{transports, Web3};

pub struct EventFetcher<T>
where
    T: EventDb,
{
    interval: Interval,
    web3: Web3<transports::Http>,
    address: Address,
    abi: Vec<Event>,
    db: T,
}

impl<T> EventFetcher<T>
where
    T: EventDb,
{
    pub fn new(web3: Web3<transports::Http>, address: Address, abi: Vec<Event>, db: T) -> Self {
        EventFetcher {
            interval: Interval::new_interval(Duration::from_secs(1)),
            address,
            abi,
            web3,
            db,
        }
    }

    fn filter_logs(&self, event: &Event, logs: Vec<RawLog>) -> Vec<RawLog> {
        if let Some(last_logged_block) = self.db.get_last_logged_block(event.signature()) {
            logs.iter()
                .filter(|&log| {
                    if let Some(n) = log.block_number {
                        n.low_u64() > last_logged_block
                    } else {
                        false
                    }
                })
                .cloned()
                .collect()
        } else {
            logs.clone()
        }
    }

    fn decode_params(&self, event: &Event, log: &RawLog) -> Result<Vec<DecodedParam>, Error> {
        let event_params = &event.inputs;
        if event_params.is_empty() {
            return Ok(vec![]);
        }

        let result = decode(
            &event_params
                .iter()
                .map(|event_param| event_param.kind.clone())
                .collect::<Vec<ParamType>>(),
            &log.data.0,
        );

        match result {
            Ok(mut tokens) => {
                // In order to `pop` in order from the first element, reverse the tokens.
                tokens.reverse();
                event_params
                    .iter()
                    .map(|ep| -> Result<DecodedParam, Error> {
                        if let Some(token) = tokens.pop() {
                            Ok(DecodedParam {
                                event_param: ep.clone(),
                                token,
                            })
                        } else {
                            Err(ErrorKind::InvalidData.into())
                        }
                    })
                    .collect()
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub log: RawLog,
    pub event_signature: H256,
    pub params: Vec<DecodedParam>,
}

#[derive(Debug, Clone)]
pub struct DecodedParam {
    pub event_param: EventParam,
    pub token: Token,
}

impl<T> Stream for EventFetcher<T>
where
    T: EventDb,
{
    type Item = Vec<Log>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Vec<Log>>, ()> {
        try_ready!(self.interval.poll().map_err(|_| ()));
        let mut all_logs: Vec<Log> = vec![];

        for event in self.abi.iter() {
            let sig = event.signature();
            let from_block: u64 = match self.db.get_last_logged_block(sig) {
                Some(n) => n,
                None => 0,
            };
            let filter = FilterBuilder::default()
                .address(vec![self.address])
                .from_block(BlockNumber::Number(from_block))
                .topic_filter(TopicFilter {
                    topic0: Topic::This(event.signature()),
                    topic1: Topic::Any,
                    topic2: Topic::Any,
                    topic3: Topic::Any,
                })
                .build();

            match self.web3.eth().logs(filter).wait().map_err(|e| e) {
                Ok(v) => {
                    let decoded: Result<Vec<Log>, Error> = self
                        .filter_logs(event, v)
                        .iter()
                        .map(|raw_log| -> Result<Log, Error> {
                            match self.decode_params(event, raw_log) {
                                Ok(decoded_params) => Ok(Log {
                                    log: raw_log.clone(),
                                    event_signature: event.signature(),
                                    params: decoded_params,
                                }),
                                Err(e) => Err(e),
                            }
                        })
                        .collect();

                    match decoded {
                        Ok(logs) => {
                            if let Some(last_log) = logs.last() {
                                if let Some(block_num) = last_log.log.block_number {
                                    self.db.set_last_logged_block(sig, block_num.low_u64());
                                };
                            };

                            all_logs.extend_from_slice(&logs);
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            };
        }

        Ok(Async::Ready(Some(all_logs)))
    }
}

pub trait EventHandler {
    fn on_event(&self, log: &Log);
}

pub struct EventWatcher<T, E>
where
    T: EventDb,
    E: EventHandler,
{
    stream: EventFetcher<T>,
    handler: E,
    _eloop: transports::EventLoopHandle,
}

impl<T, E> EventWatcher<T, E>
where
    T: EventDb,
    E: EventHandler,
{
    pub fn new(url: &str, address: Address, abi: Vec<Event>, db: T, handler: E) -> Self {
        let (eloop, transport) = web3::transports::Http::new(url).unwrap();
        let web3 = web3::Web3::new(transport);
        let stream = EventFetcher::new(web3, address, abi, db);

        EventWatcher {
            _eloop: eloop,
            stream,
            handler,
        }
    }
}

impl<T, E> Future for EventWatcher<T, E>
where
    T: EventDb,
    E: EventHandler,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let logs = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => continue,
            };

            for log in logs.iter() {
                self.handler.on_event(&log);
            }
        }
    }
}
