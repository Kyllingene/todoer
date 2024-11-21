use std::hash::{Hash, DefaultHasher, Hasher};
use std::path::Path;

#[macro_export]
macro_rules! join {
    ($sep:literal, $($arg:literal),+ $(,)?) => {
        $crate::join!($($sep $arg)*)
    };

    ($_:literal $($args:literal)*) => { concat!($($args),*) };
}

pub fn get_hash(data: &Path) -> u128 {
    let mut state = DefaultHasher::new();
    data.hash(&mut state);
    let base = state.finish();

    // for fun (and profit)
    let mut first = [0; 8];
    let bytes = data.as_os_str().as_encoded_bytes();
    let cap = bytes.len().min(first.len());
    first[..cap].copy_from_slice(&bytes[..cap]);
    let first = u64::from_ne_bytes(first) as u128;

    ((base as u128) << 64) | (bytes.len() as u128 ^ first)
}
