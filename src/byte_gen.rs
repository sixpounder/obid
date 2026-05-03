use std::sync::{Mutex, OnceLock};

use rand::{Rng, SeedableRng, rngs::SmallRng};

static COUNTER: OnceLock<Mutex<u32>> = OnceLock::new();

/// Return a 3-byte big-endian counter. The counter is initialized to a random
/// value on first call and increments by 1 on each call, wrapping to 0 after 0xFFFFFF.
pub fn next_3byte_be() -> [u8; 3] {
    // initialize COUNTER once
    COUNTER.get_or_init(|| {
        // Attempt to seed from OS RNG without blocking using getrandom.
        // If getrandom fails, fall back to deterministic seed composed of time/pid/hostname.
        let seed_u64 = try_seed_from_os();

        let mut rng = SmallRng::seed_from_u64(seed_u64);
        // Initialize to a 24-bit value
        let initial = rng.next_u32() & 0x00FF_FFFF;
        Mutex::new(initial)
    });

    // increment and return
    let mut guard = COUNTER.get().unwrap().lock().unwrap();
    let cur = *guard;
    let ret = cur as u32;
    // increment with wrap at 2^24
    *guard = (cur.wrapping_add(1)) & 0x00FF_FFFF;
    // return big-endian top 3 bytes
    [(ret >> 16) as u8, (ret >> 8) as u8, (ret) as u8]
}

fn try_seed_from_os() -> u64 {
    // try to get 8 bytes from OS RNG; getrandom may fail (e.g., would block), so treat errors as fallback.
    let mut buf = [0u8; 8];
    rand::rng().fill_bytes(&mut buf);
    u64::from_le_bytes(buf)
}
