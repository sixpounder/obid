use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{Rng, SeedableRng, rngs::SmallRng};

static COUNTER: OnceLock<Mutex<u32>> = OnceLock::new();

/// Return a 3-byte big-endian counter. The counter is initialized to a random
/// value on first call and increments by 1 on each call, wrapping to 0 after 0xFFFFFF.
pub(crate) fn next_3byte_be() -> [u8; 3] {
    // initialize COUNTER once
    COUNTER.get_or_init(|| {
        // Attempt to seed from OS RNG without blocking using getrandom.
        // If getrandom fails, fall back to deterministic seed composed of time/pid/hostname.
        let seed_u64 = match try_seed_from_os() {
            Some(s) => s,
            None => deterministic_seed(),
        };
        // Initialize to a 24-bit value
        let mut rng = SmallRng::seed_from_u64(seed_u64);
        let initial = rng.next_u32() & 0x00FF_FFFF;
        Mutex::new(initial)
    });

    // increment and return
    let mut guard = COUNTER.get().unwrap().lock().unwrap();
    let cur = *guard;
    // increment with wrap at 2^24
    *guard = (cur.wrapping_add(1)) & 0x00FF_FFFF;
    // return big-endian top 3 bytes
    [(cur >> 16) as u8, (cur >> 8) as u8, (cur) as u8]
}

fn try_seed_from_os() -> Option<u64> {
    let mut buf = [0u8; 8];
    if getrandom::fill(&mut buf).is_ok() {
        Some(u64::from_le_bytes(buf))
    } else {
        None
    }
}

fn deterministic_seed() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let hostname = get_hostname_string();

    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    pid.hash(&mut hasher);
    hostname.hash(&mut hasher);
    hasher.finish()
}

fn get_hostname_string() -> String {
    if let Ok(h) = std::env::var("HOSTNAME")
        && !h.is_empty()
    {
        return h;
    }

    #[cfg(unix)]
    {
        use libc::{_SC_HOST_NAME_MAX, c_char, sysconf};
        use std::ffi::CStr;
        let max = unsafe { sysconf(_SC_HOST_NAME_MAX) } as usize;
        let buf_len = if max > 0 { max + 1 } else { 256 };
        let mut buf = vec![0u8; buf_len];
        let ptr = buf.as_mut_ptr() as *mut c_char;
        if unsafe { libc::gethostname(ptr, buf_len) } == 0
            && let Ok(s) = unsafe { CStr::from_ptr(ptr) }.to_str()
        {
            return s.to_owned();
        }
    }
    String::from("unknown-host")
}
