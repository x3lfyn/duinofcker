use sha1::{Sha1, Digest};

pub fn pow(last_h: String, exp_h: [u8; 20], diff: u64) -> u64 {
    let base = Sha1::new_with_prefix(last_h.as_bytes());
    for nonce in 0..(100 * diff + 1) {
        let mut temp = base.clone();
        temp.update(nonce.to_string().as_bytes());

        let digest = temp.finalize();
        if digest == exp_h.into() {
            return nonce
        }
    }

    return 0u64
}
