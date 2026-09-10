/**
 * The Tagma security layer: authority, integrity, audit, and channel, with
 * the legacy and the tagma-sec (Delos) implementations, the
 * {@link org.ssccs.syntagma.sec.SecStack} composition, and the route-update
 * workflow.
 *
 * <p>Port of the C++ {@code tagma_sec} module in {@code sw/cpp/tagma_sec}; the
 * underlying behavior mirrors the Rust {@code tagma-sec} crate in
 * {@code sw/rust/sec}.
 *
 * <p>Hash choice, a deliberate porting difference: the Rust crate uses blake3
 * keyed hashing. The C++ port uses SHA-256 for payload commitments and
 * RFC 2104 HMAC-SHA-256 for keyed tags, and documents that choice in
 * {@code tagma_sec/hash.h}. Java mirrors the C++ implementation through the
 * JDK ({@code java.security.MessageDigest} with {@code SHA-256},
 * {@code javax.crypto.Mac} with {@code HmacSHA256}) and produces the same
 * bytes, so the C++ vectors hold unchanged and the tags match any standard
 * HMAC-SHA-256 implementation. See
 * {@link org.ssccs.syntagma.sec.Hashes}.
 *
 * <p>Type mapping: {@code PrincipalId} and {@code Epoch} (C++ {@code uint64_t})
 * map to Java {@code long}; {@code Action} (C++ {@code uint8_t}) maps to
 * {@code int}, with {@link org.ssccs.syntagma.sec.Authority#ACTION_ROUTE_UPDATE}
 * as the single workflow action; {@code Path} (C++ {@code std::vector<Coord>},
 * Rust {@code Vec<Coord>}) maps to
 * {@link org.ssccs.syntagma.core.CoordPath}, the core coordinate sequence both
 * references alias, so the port reuses the core type rather than wrapping it;
 * {@code std::optional} maps to {@code Optional} and {@code OptionalLong};
 * {@code std::array<uint8_t, 32>} maps to {@code byte[]} validated to 32 bytes
 * at construction, copied on the way in and cloned on the way out.
 */
package org.ssccs.syntagma.sec;
