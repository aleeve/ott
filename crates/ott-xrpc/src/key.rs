use ed25519_dalek::SigningKey;
use elliptic_curve::rand_core::OsRng;

fn multikey_ed25519(key: &[u8]) -> String {
    let mut buf = vec![0xED, 0x01]; // Ed25519-pub multicodec (varint encoded)
    buf.extend_from_slice(key);
    multibase::encode(multibase::Base::Base58Btc, buf)
}

pub fn generate_key() -> String {
    // Generate keypair
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.as_bytes();
    multikey_ed25519(public_key_bytes)
}
