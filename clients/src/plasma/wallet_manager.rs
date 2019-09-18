use super::wallet_db::WalletDb;
use bytes::Bytes;
use ethsign::SecretKey as EthSecretKey;
use plasma_db::traits::kvs::KeyValueStore;
use rand::*;
use secp256k1::SecretKey;

pub struct WalletManager<'a, KVS> {
    db: WalletDb<'a, KVS>,
}

impl<'a, KVS: KeyValueStore> WalletManager<'a, KVS> {
    pub fn new(kvs: &'a KVS) -> Self {
        Self {
            db: WalletDb::new(kvs),
        }
    }

    pub fn generate_key_session(&mut self) -> (Bytes, EthSecretKey) {
        let mut rnd = rand::thread_rng();
        let secret_key_raw = SecretKey::random(&mut rnd).serialize();
        let session_raw = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let session = Bytes::from(session_raw);
        let _ = self.db.put_private_key(&session, &secret_key_raw); // TODO: error handling
        let secret_key = EthSecretKey::from_raw(&secret_key_raw).unwrap();

        (session, secret_key)
    }

    pub fn get_key(&self, session: &Bytes) -> Option<EthSecretKey> {
        match self
            .db
            .get_private_key(session)
            .map(|k| EthSecretKey::from_raw(&k).unwrap())
        {
            Ok(k) => Some(k),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plasma_db::{impls::kvs::memory::CoreDbMemoryImpl, traits::db::DatabaseTrait};

    #[test]
    fn test_generate_key_session() {
        let kvs = CoreDbMemoryImpl::open("test");
        let mut wallet_manager = WalletManager::new(&kvs);
        let (session, private_key) = wallet_manager.generate_key_session();
        assert_eq!(
            wallet_manager
                .get_key(&session)
                .unwrap()
                .public()
                .bytes()
                .to_vec(),
            private_key.public().bytes().to_vec()
        );
    }
}
