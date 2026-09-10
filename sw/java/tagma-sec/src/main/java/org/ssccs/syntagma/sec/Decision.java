package org.ssccs.syntagma.sec;

/**
 * Authorization decision, mirroring the C++ {@code Decision} enum class in
 * {@code sw/cpp/tagma_sec/include/tagma_sec/types.h} and the Rust
 * {@code Decision} enum in {@code sw/rust/sec/src/types.rs}.
 *
 * <p>The C++ spellings {@code Decision::Allow} and {@code Decision::Deny} map
 * to the constants {@code ALLOW} and {@code DENY}.
 */
public enum Decision {

    /** The attestation authorizes the path for the action at the epoch. */
    ALLOW,

    /** The attestation does not authorize the path for the action at the epoch. */
    DENY
}
