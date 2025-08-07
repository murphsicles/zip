use secrecy::Secret;
use uuid::Uuid;
use zip::storage::ZipStorage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_get_user_data() {
        let storage = ZipStorage::new().unwrap();
        let user_id = Uuid::new_v4();
        let data = b"test_data";
        storage.store_user_data(user_id, data).unwrap();
        let retrieved = storage.get_user_data(user_id).unwrap().unwrap();
        assert_eq!(&retrieved[..], data);
    }

    #[test]
    fn test_store_get_private_key() {
        let storage = ZipStorage::new().unwrap();
        let key = Secret::new(vec![1, 2, 3]);
        storage.store_private_key(key.clone()).unwrap();
        let retrieved = storage.get_private_key().unwrap();
        assert_eq!(retrieved.expose_secret(), key.expose_secret());
    }

    #[test]
    fn test_cache_get_utxos() {
        let storage = ZipStorage::new().unwrap();
        let user_id = Uuid::new_v4();
        let utxos = b"utxo_data";
        storage.cache_utxos(user_id, utxos).unwrap();
        let retrieved = storage.get_utxos(user_id).unwrap().unwrap();
        assert_eq!(&retrieved[..], utxos);
    }
}
