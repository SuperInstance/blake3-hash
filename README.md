# blake3-hash

A teaching-grade BLAKE3-inspired hash function implementing the core compression, mixing, and streaming architecture of the BLAKE3 cryptographic hash in pure Rust. This is a **simplified reference** — not cryptographically suitable — designed to demonstrate chunk-based binary-tree hashing, the G function, and domain-separated compression.

## Why It Matters

BLAKE3 is the fastest production hash function in widespread use (2020–present), achieving >10× the throughput of SHA-256 on modern CPUs by combining:

- **Merkle-tree parallelism** — each 1024-byte chunk is independent
- **Binary tree fusion** — chunks combine via chaining values
- **SIMD-friendly G rounds** — 7 rounds of column + diagonal mixing

Understanding the internals matters for cryptographic engineering, content-addressed storage, and systems that need tree-hashing semantics (e.g., IPFS, Bazel remote caching). This crate isolates those mechanics without the complexity of the full BLAKE3 spec.

## How It Works

### Compression Function

The core is `compress(cv, block, counter, flags) → [u32; 8]`, which maps a 64-byte block and 8-word chaining value to a new 8-word chaining value:

$$h' = \text{compress}(cv, \text{block}, t, f)$$

The 16-word state matrix is initialized as `[cv | block_words]`, then the counter is XOR'd into positions 8–9. After 7 rounds of mixing, the output is computed as:

$$\text{out}[i] = cv[i] \oplus h_i \oplus h_{i+8}$$

### The G Function

Each round applies 8 calls to the G function — 4 on columns, 4 on diagonals — using the permutation `[2,6,3,10,7,0,4,13,1,11,12,5,9,14,15,8]`:

```
G(a, b, c, d, x, y):
    a = a + x + b;  d = (d ⊕ a) >>> 16
    c = c + d;      b = (b ⊕ c) >>> 12
    a = a + y + b;  d = (d ⊕ a) >>> 8
    c = c + d;      b = (b ⊕ c) >>> 7
```

### Streaming Model

Input is buffered into 1024-byte chunks. Each chunk is compressed block-by-block (16 × 64-byte blocks per chunk). The chaining value stack grows as chunks complete:

| Stage | Operation | Complexity |
|-------|-----------|------------|
| `update()` | Buffer + compress chunks | O(n) time, O(1) extra space |
| `finalize()` | Pad + compress remainder | O(1) |
| Total hash | n bytes → 32 bytes | **O(n)** |

### Big-O Summary

- **Time**: O(n) — linear in input size, single-pass streaming
- **Space**: O(log n) — chaining value stack depth = tree height
- **Parallelism**: O(n/1024) chunks are independent (exploited by real BLAKE3)

## Quick Start

```rust
use blake3_hash::{blake3_hash, Blake3Hasher};

// One-shot
let digest = blake3_hash(b"hello blake3 world");
assert_eq!(digest.len(), 32);

// Streaming
let mut h = Blake3Hasher::new();
h.update(b"hello ");
h.update(b"blake3 ");
h.update(b"world");
let digest2 = h.finalize();
```

## API

| Type / Function | Description |
|-----------------|-------------|
| `Blake3Hasher::new()` | Create a streaming hasher with the BLAKE3 IV |
| `Blake3Hasher::update(&[u8])` | Absorb bytes into the hash state |
| `Blake3Hasher::finalize() → [u8; 32]` | Produce the 256-bit digest |
| `blake3_hash(&[u8]) → [u8; 32]` | One-shot convenience function |

## Architecture Notes

The design enforces the **γ + η = C** principle: the compression function (γ) transforms state, while the XOR-folding at output (η) ensures that the chaining value always influences the result. Their combination yields the conservation invariant C — the property that every bit of output depends on every bit of input via the avalanche of G rounds.

The chaining value stack is the physical embodiment of this invariant: each push represents a chunk boundary, and the tree structure ensures that no chunk can be modified without changing every downstream chaining value.

## References

- Aumasson, J.-P., O'Connor, D., & Sue-Carisma, S. (2020). *BLAKE3*. GitHub: <https://github.com/BLAKE3-team/BLAKE3-specs>
- Aumasson, J.-P., Henzen, L., Meier, W., & Naya-Plasencia, M. (2014). *BLAKE2: Simpler, Smaller, Fast as MD5*. IACR ePrint 2013/322.
- Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2011). *The Keccak sponge function family*. BLAKE3 borrows the tree-mode concept.
- NIST FIPS 180-4 (2015). *Secure Hash Standard*. SHA-2 comparison baseline.

## License

MIT
