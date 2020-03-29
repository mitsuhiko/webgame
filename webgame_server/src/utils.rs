use rand::{seq::SliceRandom, thread_rng};

const CHARS: &[u8; 22] = b"BCDFGHJKLMNPQRSTUVWXZY";

pub fn generate_join_code() -> String {
    let mut rng = thread_rng();
    (0..6)
        .map(|_| *CHARS.choose(&mut rng).unwrap() as char)
        .collect()
}
