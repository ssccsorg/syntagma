# base11172

Tagma native serialization format: self-validating, printable encoding of
arbitrary byte sequences through the compositional character space.

The alphabet is the 11,172 valid Tagma coordinates (the Hangul syllable
block U+AC00..U+D7A3). Every coordinate index maps to exactly one
character, and a pair of characters encodes one 16-bit value. Because the
alphabet is the Tagma space, the 54,364 remaining code points are invalid
by construction, so a corrupted character is detected at decode time
without any checksum.

## Properties

- Self-validating: characters outside U+AC00..U+D7A3 are rejected by
  `decode_pair` and `decode_bytes`.
- No special characters: the alphabet needs no escaping in URLs, JSON
  strings, or shell arguments.
- Deterministic: encoding is a pure function of the input.
- `no_std` compatible: the crate requires only `alloc`.

## Usage

```rust
use base11172::{encode_bytes, decode_bytes};

let data = b"Base11172!";
let encoded = encode_bytes(data);
let decoded = decode_bytes(&encoded).unwrap();
assert_eq!(&decoded[..], data);
```

The CLI offers the same operations:

```text
base11172 encode <text>     Encode text to Base11172
base11172 decode <string>   Decode Base11172 back to text
base11172 bench             Compare density vs Base64
```

## Encoding scheme

A 16-bit value splits into a high part and a low part against the alphabet
size (11,172); each part becomes a `Coord` and then a character:

```text
value = hi * 11172 + lo,  each of hi and lo in [0, 11172)
```

Byte sequences are encoded two bytes per character pair, little-endian.
The format is pair-oriented: an odd-length input gains one trailing zero
byte after a round trip, so callers that need exact round trips on
odd-length data should length-prefix the input.

At the character level the format is denser than Base64 (one character per
byte against 1.333), at the cost of UTF-8 size (three bytes per character).

## Status

Crate version 0.1.0, Apache-2.0, a workspace member of the syntagma Rust
workspace. Covered by round-trip tests over `u16` extremes, byte strings,
all 256 byte values, and invalid-character rejection. A self-contained C++
port mirrors the same interface under `sw/cpp/base11172`.
