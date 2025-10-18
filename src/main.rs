use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

// Base64 URL-safe alphabet ordered by ASCII value for lexical sorting
// This ensures that encoded strings maintain chronological order
const BASE64_ALPHABET: &[u8; 64] = b"-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

/// Encodes a u64 value as Base64 URL-safe string (11 characters, no padding)
fn base64_encode_u64(value: u64) -> String {
    let bytes = value.to_be_bytes(); // Big-endian encoding
    let mut result = Vec::with_capacity(11);

    // Process bytes in groups of 3 (24 bits) -> 4 base64 chars
    let mut i = 0;
    while i + 2 < bytes.len() {
        let b1 = bytes[i] as usize;
        let b2 = bytes[i + 1] as usize;
        let b3 = bytes[i + 2] as usize;

        result.push(BASE64_ALPHABET[(b1 >> 2) & 0x3F]);
        result.push(BASE64_ALPHABET[((b1 & 0x03) << 4) | ((b2 >> 4) & 0x0F)]);
        result.push(BASE64_ALPHABET[((b2 & 0x0F) << 2) | ((b3 >> 6) & 0x03)]);
        result.push(BASE64_ALPHABET[b3 & 0x3F]);

        i += 3;
    }

    // Handle remaining bytes (2 bytes left for 8-byte u64)
    if i < bytes.len() {
        let b1 = bytes[i] as usize;
        result.push(BASE64_ALPHABET[(b1 >> 2) & 0x3F]);

        if i + 1 < bytes.len() {
            let b2 = bytes[i + 1] as usize;
            result.push(BASE64_ALPHABET[((b1 & 0x03) << 4) | ((b2 >> 4) & 0x0F)]);
            result.push(BASE64_ALPHABET[(b2 & 0x0F) << 2]);
        } else {
            result.push(BASE64_ALPHABET[(b1 & 0x03) << 4]);
        }
    }

    String::from_utf8(result).unwrap()
}

// Thread-local state for sequence tracking
thread_local! {
    static LAST_TIMESTAMP_MS: Cell<u64> = Cell::new(0);
    static SEQUENCE: Cell<u16> = Cell::new(0);
}

/// Get current Unix timestamp in milliseconds
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_millis() as u64
}

/// Generate a 10-bit random value using RandomState
fn generate_random_10bit() -> u16 {
    let random_state = RandomState::new();
    let mut hasher = random_state.build_hasher();

    // Add some entropy from current time nanos
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    hasher.write_u32(nanos);
    let hash = hasher.finish();

    // Take lower 10 bits
    (hash & 0x3FF) as u16
}

/// Spin-wait until the next millisecond
fn wait_next_millisecond(current: u64) {
    while current_time_ms() == current {
        std::hint::spin_loop();
    }
}

/// Generate a Tiny64 ID
pub fn generate_tiny64() -> String {
    let (timestamp_ms, sequence, random) = LAST_TIMESTAMP_MS.with(|last_time| {
        SEQUENCE.with(|seq| {
            let mut now = current_time_ms();
            let last = last_time.get();
            let mut current_seq = seq.get();

            if now == last {
                // Same millisecond: increment sequence
                current_seq = (current_seq + 1) % 4096;

                if current_seq == 0 {
                    // Sequence overflow: wait for next millisecond
                    wait_next_millisecond(now);
                    now = current_time_ms();
                }
            } else {
                // New millisecond: reset sequence
                current_seq = 0;
            }

            // Update state
            last_time.set(now);
            seq.set(current_seq);

            // Generate random 10-bit value
            let random = generate_random_10bit();

            (now, current_seq, random)
        })
    });

    // Construct 64-bit value:
    // [ 42 bits: timestamp_ms ] [ 12 bits: sequence ] [ 10 bits: random ]
    let value = ((timestamp_ms & 0x3FF_FFFF_FFFF) << 22)
        | ((sequence as u64 & 0xFFF) << 10)
        | (random as u64 & 0x3FF);

    base64_encode_u64(value)
}

fn print_help() {
    println!("Tiny64 - Time-Ordered Compact Unique IDs");
    println!();
    println!("USAGE:");
    println!("    tiny64       Generate a single Tiny64 ID");
    println!("    tiny64 -h    Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Tiny64 is a compact 64-bit identifier format designed for high-performance");
    println!("    systems that require time-sortable unique IDs with low collision probability");
    println!("    and efficient generation.");
    println!();
    println!("FEATURES:");
    println!("    - Short: Only 11 characters (Base64 URL-safe)");
    println!("    - Time-sortable: IDs sort chronologically as strings");
    println!("    - Low collision rate: Timestamp + sequence + randomness");
    println!("    - Fast generation: Suitable for shell scripts or lightweight services");
    println!("    - Distributed safe: Works in multi-process environments");
    println!("    - Zero external dependencies");
    println!();
    println!("FORMAT:");
    println!("    [ 42 bits: timestamp (ms since Unix epoch) ]");
    println!("    [ 12 bits: sequence number                ]");
    println!("    [ 10 bits: randomness                     ]");
    println!();
    println!("EXAMPLES:");
    println!("    $ tiny64");
    println!("    Obrl8O3--Cw");
    println!();
    println!("    $ for i in {{1..5}}; do tiny64; done");
    println!("    Obrl8O3--Cw");
    println!("    Obrl8O3-0QB");
    println!("    Obrl8O3-19o");
    println!("    Obrl8O3-2Pw");
    println!("    Obrl8O3-3g3");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for help option
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return;
    }

    // Generate and print a single ID
    println!("{}", generate_tiny64());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_length() {
        let id = base64_encode_u64(0x123456789ABCDEF0);
        assert_eq!(id.len(), 11);
    }

    #[test]
    fn test_generate_tiny64_format() {
        let id = generate_tiny64();
        assert_eq!(id.len(), 11);

        // Check all characters are Base64 URL-safe
        for ch in id.chars() {
            assert!(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
        }
    }

    #[test]
    fn test_multiple_ids_different() {
        let id1 = generate_tiny64();
        let id2 = generate_tiny64();
        let id3 = generate_tiny64();

        // IDs should be different (very high probability)
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_time_ordering() {
        let mut ids = vec![];
        for _ in 0..100 {
            ids.push(generate_tiny64());
        }

        // IDs should be in lexical order (or equal if same millisecond)
        for i in 0..ids.len() - 1 {
            if ids[i] > ids[i + 1] {
                eprintln!("Order violation at index {}: '{}' > '{}'", i, ids[i], ids[i + 1]);
            }
            assert!(ids[i] <= ids[i + 1]);
        }
    }

    #[test]
    fn test_debug_values() {
        // Generate a few IDs and print raw values
        for _ in 0..5 {
            let id = generate_tiny64();
            println!("Generated ID: {}", id);
        }
    }
}
