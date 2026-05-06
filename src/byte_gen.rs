use core::{
    hash::{BuildHasher, Hash, Hasher},
    sync::atomic::AtomicU32,
};

use rand::{Rng, SeedableRng, rngs::SmallRng};

#[cfg(feature = "std")]
static COUNTER: std::sync::OnceLock<AtomicU32> = std::sync::OnceLock::new();

#[cfg(not(feature = "std"))]
static COUNTER: once_cell::sync::OnceCell<AtomicU32> = once_cell::sync::OnceCell::new();

/// Return a 3-byte big-endian counter. The counter is initialized to a random
/// value on first call and increments by 1 on each call, wrapping to 0 after 0xFFFFFF.
pub(crate) fn next_3byte_be() -> [u8; 3] {
    // initialize COUNTER once
    let counter = COUNTER.get_or_init(|| {
        // Attempt to seed from OS RNG without blocking using getrandom.
        // If getrandom fails, fall back to deterministic seed composed of time/pid/hostname.
        let seed_u64 = match try_seed_from_os() {
            Some(s) => s,
            None => deterministic_seed(),
        };
        // Initialize to a 24-bit value
        let mut rng = SmallRng::seed_from_u64(seed_u64);
        let initial = rng.next_u32() & 0x00FF_FFFF;
        AtomicU32::new(initial)
    });

    // increment and return
    let prev = counter.fetch_add(1, core::sync::atomic::Ordering::Release); // atomic, wraps
    let new = prev.wrapping_add(1); // new value (wrapped)
    // return big-endian top 3 bytes
    [(new >> 16) as u8, (new >> 8) as u8, (new) as u8]
}

/// Attempts to seed the RNG from the OS's RNG using getrandom,
/// falling back to a deterministic seed if unavailable.
fn try_seed_from_os() -> Option<u64> {
    let mut buf = [0u8; 8];
    if getrandom::fill(&mut buf).is_ok() {
        Some(u64::from_le_bytes(buf))
    } else {
        None
    }
}

/// Returns a deterministic seed based on the current time, process ID, and hostname.
fn deterministic_seed() -> u64 {
    let now = time::OffsetDateTime::now_utc().nanosecond();
    let pid = getpid();
    let hostname = get_hostname_string();

    let hasher = hashbrown::DefaultHashBuilder::default();
    let mut hasher = hasher.build_hasher();
    now.hash(&mut hasher);
    pid.hash(&mut hasher);
    hostname.hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "std")]
fn getpid() -> u32 {
    std::process::id()
}

#[cfg(not(feature = "std"))]
fn getpid() -> u32 {
    0
}

/// Returns the hostname as a string, using the `HOSTNAME` environment variable if set,
/// or the system's hostname if available.
#[cfg(feature = "std")]
fn get_hostname_string() -> std::ffi::OsString {
    gethostname::gethostname()
}

#[cfg(not(feature = "std"))]
fn get_hostname_string() -> &'static str {
    "unknown"
}
