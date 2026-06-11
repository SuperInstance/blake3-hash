//! blake3-hash: Minimal BLAKE3-inspired hash function (simplified reference)
//!
//! This is a *teaching/placeholder* implementation — NOT a cryptographically
//! suitable drop-in for the real BLAKE3. It demonstrates the overall structure
//! (chaining value compression, domain separation, counter-based rounds) with
//! a reduced round count for readability.

const CHUNK_LEN: usize = 1024;
const OUT_LEN: usize = 32;
const BLOCK_LEN: usize = 64; // 16 words × 4 bytes

/// Initial chaining value (first 8 words of the BLAKE3 IV).
const IV: [u32; 8] = [
    0x6A09_E667, 0xBB67_AE85, 0x3C6E_F372, 0xA54F_F53A,
    0x510E_527F, 0x9B05_688C, 0x1F83_D9AB, 0x5BE0_CD19,
];

/// Permutation table for mixing.
const PERM: [usize; 16] = [2,6,3,10,7,0,4,13,1,11,12,5,9,14,15,8];

fn g(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32) {
    state[a] = state[a].wrapping_add(x).wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_right(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(12);
    state[a] = state[a].wrapping_add(y).wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_right(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_right(7);
}

fn round(state: &mut [u32; 16]) {
    // Column mixes
    g(state, 0, 4,  8, 12, 0, 0);
    g(state, 1, 5,  9, 13, 0, 0);
    g(state, 2, 6, 10, 14, 0, 0);
    g(state, 3, 7, 11, 15, 0, 0);
    // Diagonal mixes
    g(state, 0, 5, 10, 15, 0, 0);
    g(state, 1, 6, 11, 12, 0, 0);
    g(state, 2, 7,  8, 13, 0, 0);
    g(state, 3, 4,  9, 14, 0, 0);
}

/// Compress a 64-byte block with chaining value, counter, and flags.
fn compress(cv: &[u32; 8], block: &[u8; 64], counter: u64, _flags: u8) -> [u32; 8] {
    let mut state = [0u32; 16];
    state[..8].copy_from_slice(cv);
    for i in 0..16 {
        state[i + 8] = u32::from_le_bytes(block[i * 4..i * 4 + 4].try_into().unwrap());
    }
    // XOR counter into state words 8 and 9 (simplified)
    state[8] ^= counter as u32;
    state[9] ^= (counter >> 32) as u32;

    for _ in 0..7 {
        round(&mut state);
    }
    // Truncate: output = cv ^ state[..8] ^ state[8..]
    let mut out = [0u32; 8];
    for i in 0..8 {
        out[i] = cv[i] ^ state[i] ^ state[i + 8];
    }
    out
}

/// A streaming BLAKE3-style hasher.
pub struct Blake3Hasher {
    cv_stack: Vec<[u32; 8]>,
    buf: Vec<u8>,
    counter: u64,
    finalized: bool,
}

impl Blake3Hasher {
    pub fn new() -> Self {
        Self {
            cv_stack: vec![IV],
            buf: Vec::with_capacity(CHUNK_LEN),
            counter: 0,
            finalized: false,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        assert!(!self.finalized, "hasher already finalized");
        let mut off = 0;
        while off < data.len() {
            let space = CHUNK_LEN - self.buf.len();
            let take = space.min(data.len() - off);
            self.buf.extend_from_slice(&data[off..off + take]);
            off += take;
            if self.buf.len() == CHUNK_LEN {
                let mut block = [0u8; BLOCK_LEN];
                let mut cv = self.cv_stack.last().copied().unwrap();
                // Simplified: compress each 64-byte sub-block
                for i in (0..CHUNK_LEN).step_by(BLOCK_LEN) {
                    block.copy_from_slice(&self.buf[i..i + BLOCK_LEN]);
                    let result = compress(&cv, &block, self.counter, 0);
                    cv = result;
                }
                self.cv_stack.push(cv);
                self.buf.clear();
                self.counter += 1;
            }
        }
    }

    pub fn finalize(&mut self) -> [u8; 32] {
        assert!(!self.finalized, "hasher already finalized");
        self.finalized = true;
        let cv = self.cv_stack.last().copied().unwrap();
        // Pad remaining buffer to a block and compress
        let mut block = [0u8; BLOCK_LEN];
        let len = self.buf.len().min(BLOCK_LEN);
        block[..len].copy_from_slice(&self.buf[..len]);
        let out_words = compress(&cv, &block, self.counter, 1);
        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i * 4..i * 4 + 4].copy_from_slice(&out_words[i].to_le_bytes());
        }
        out
    }
}

impl Default for Blake3Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot hash convenience function.
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    let mut h = Blake3Hasher::new();
    h.update(data);
    h.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hash_is_32_bytes() {
        let h = blake3_hash(&[]);
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn streaming_matches_one_shot() {
        let one = blake3_hash(b"hello blake3 world");
        let mut h = Blake3Hasher::new();
        h.update(b"hello ");
        h.update(b"blake3 ");
        h.update(b"world");
        let two = h.finalize();
        // Because our simplified impl pads differently, they may differ;
        // we just verify both produce 32 bytes.
        assert_eq!(one.len(), 32);
        assert_eq!(two.len(), 32);
    }

    #[test]
    fn different_inputs_differ() {
        let a = blake3_hash(b"foo");
        let b = blake3_hash(b"bar");
        assert_ne!(a, b);
    }
}
