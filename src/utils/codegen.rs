use base64::Engine as _;
use rand_core::{OsRng, TryRngCore};

pub fn gen_code() -> String {
    let mut buf = [0u8; 9];
    OsRng.try_fill_bytes(&mut buf).expect("OsRng failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}
