use odht::{Config, FxHashFn, HashTable, HashTableOwned};

struct MyConfig;

impl Config for MyConfig {
    type Key = u64;
    type Value = u32;
    type EncodedKey = [u8; 8];
    type EncodedValue = [u8; 4];
    type H = FxHashFn;

    #[inline]
    fn encode_key(k: &Self::Key) -> Self::EncodedKey {
        k.to_le_bytes()
    }

    #[inline]
    fn encode_value(v: &Self::Value) -> Self::EncodedValue {
        v.to_le_bytes()
    }

    #[inline]
    fn decode_key(k: &Self::EncodedKey) -> Self::Key {
        u64::from_le_bytes(*k)
    }

    #[inline]
    fn decode_value(v: &Self::EncodedValue) -> Self::Value {
        u32::from_le_bytes(*v)
    }
}

fn main() {
    let mut builder = HashTableOwned::<MyConfig>::with_capacity(3, 95);
    builder.insert(&1, &2);
    builder.insert(&3, &4);
    builder.insert(&5, &6);

    let serializd = builder.raw_bytes().to_owned();

    let table = HashTable::<MyConfig, &[u8]>::from_raw_bytes(&serializd[..]).unwrap();

    assert_eq!(table.get(&1), Some(2));
    assert_eq!(table.get(&3), Some(4));
    assert_eq!(table.get(&5), Some(6));
}
