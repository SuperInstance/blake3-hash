# BLAKE3 Hash

**A Rust library implementing a BLAKE3-style hash function** — a simplified, pedagogical reference implementation that demonstrates the architecture of BLAKE3 (chaining values, domain separation, counter-based chunking, G-function mixing) without the full complexity of the production specification.

## Why It Matters

BLAKE3 is the fastest cryptographic hash function in production — processing data at over 1 GB/s per core on modern hardware. It's used by Apple's Spotlight, the Bazel build system, IPFS, and Solana. Its speed comes from:

1. **Tree mode** — BLAKE3 hashes data in 1 KB chunks, then combines chunk hashes in a binary tree. This enables parallelism (each subtree is independent) and incremental/streaming hashing.
2. **G-function mixing** — Each round applies 8 G operations (4 column + 4 diagonal) inspired by BLAKE2 and ChaCha. These provide diffusion: every output bit depends on every input bit after enough rounds.
3. **Reduced rounds** — BLAKE3 uses 7 rounds (vs. BLAKE2's 12), trading a small security margin for substantial speed.

BLAKE3 is a Merkle tree inside: each 1 KB chunk is hashed independently, and the chunk hashes are combined pairwise up to a root. This means you can hash a 1 GB file in parallel across 1M chunks.

## How It Works

**Compression function**: Takes an 8-word chaining value (CV), a 64-byte block, a counter, and flags. The 16-word state is initialized with the CV in positions 0–7 and block words in 8–15. The counter is XOR'd into state words 8–9. Seven rounds of G-function mixing follow.

**G function**: A quarter-round that mixes four state words using additions, XORs, and rotations by 16, 12, 8, and 7 bits — the same pattern as BLAKE2 and ChaCha. These rotations provide diffusion across word boundaries.

**Streaming**: The `Blake3Hasher` accumulates data in a 1 KB buffer. When full, it compresses the chunk (processing 16 × 64-byte sub-blocks), pushes the resulting CV onto a stack, and resets the buffer. On `finalize()`, the remaining buffer is padded and compressed with a domain-separation flag.

**Initialization vector**: Uses the first 8 words of the SHA-256/BLAKE2 IV (e.g., `0x6A09E667`).

## Quick Start

```rust
use blake3_hash::{blake3_hash, Blake3Hasher};

// One-shot hashing
let hash = blake3_hash(b"hello world");
assert_eq!(hash.len(), 32);

// Streaming (hash data in chunks)
let mut hasher = Blake3Hasher::new();
hasher.update(b"hello ");
hasher.update(b"world");
let streamed = hasher.finalize();
assert_eq!(streamed.len(), 32);

// Different inputs produce different hashes
assert_ne!(blake3_hash(b"foo"), blake3_hash(b"bar"));
```

## API

- **`blake3_hash(data)` → `[u8; 32]`** — One-shot hash convenience function
- **`Blake3Hasher`** — Streaming hash builder
  - `new()` — Initialize with IV
  - `update(data)` — Absorb bytes
  - `finalize()` → `[u8; 32]` — Produce the final hash

## Architecture Notes

This is a pedagogical implementation for understanding the BLAKE3 architecture. For production use, prefer the official `blake3` crate which includes SIMD optimizations, assembly for ARM/x86, and the full BLAKE3 specification. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
