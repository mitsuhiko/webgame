use rand::{thread_rng, Rng};

const CHARS: &[u8; 32] = b"0123456789BCDFGHJKLMNPQRSTUVWXZY";

pub fn generate_join_code() -> String {
    let mut rng = thread_rng();
    (0..4)
        .map(|_| CHARS[rng.gen_range(0, 32)] as char)
        .collect()
}
