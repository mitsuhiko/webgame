use rand::{seq::SliceRandom, thread_rng};

const CHARS: &[u8; 30] = b"23456789BCDFGHJKLMNPQRSTUVWXZY";

pub fn generate_join_code() -> String {
    let mut rng = thread_rng();
    (0..5)
        .map(|_| *CHARS.choose(&mut rng).unwrap() as char)
        .collect()
}
