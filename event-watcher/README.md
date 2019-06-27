# Ethereum Event Watcher

## Usage
```rust
fn main() {
    let address: Address = match "e427Dbb91361bAed1B76978aF075C31dC2AB5951".parse() {
        Ok(v) => v,
        Err(e) => panic!(e),
    };

    let abi: Vec<Event> = vec![
        Event {
            name: "SetValue".to_owned(),
            inputs: vec![
                EventParam {
                    name: "key".to_owned(),
                    kind: ParamType::String,
                    indexed: false,
                },
            ],
            anonymous: false,
        }
    ];
    let db = DefaultEventDb::new();
    let mut watcher = EventWatcher::new("http://localhost:9545", address, abi, db);

    watcher.subscribe(Box::new(|log| {
        println!("{:?}", log);
    }));

    tokio::run(future::lazy(|| {
        tokio::spawn(watcher);
        Ok(())
    }));
}

```
