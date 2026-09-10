package org.ssccs.syntagma.sec;

import java.security.InvalidKeyException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Objects;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

/**
 * Keyed hashing for the tagma-sec layer: SHA-256 for payload commitments and
 * RFC 2104 HMAC-SHA-256 for keyed tags.
 *
 * <p>The Rust reference uses blake3 keyed hashing. The C++ port replaces it
 * with SHA-256 and HMAC-SHA-256 over the concatenated parts, keeping the same
 * interfaces and semantics: a 32-byte keyed commitment, tamper evidence, keyed
 * binding, and recompute-and-compare verification. The 32-byte key is a key
 * shorter than the 64-byte SHA-256 block, so HMAC zero-pads it to the block
 * length before mixing the inner and outer pads. Java mirrors the C++
 * implementation through the JDK and produces the same bytes, so tags written
 * by one port verify in the other and match any standard HMAC-SHA-256
 * implementation.
 *
 * <p>Port of the hash helpers in the C++ {@code tagma_sec/hash.h} in
 * {@code sw/cpp/tagma_sec}; the underlying behavior mirrors {@code keyed_tag}
 * in {@code sw/rust/sec/src/hash.rs} with the hash function substituted.
 */
public final class Hashes {

    /** Length of every tag, key, and payload commitment in this layer. */
    public static final int TAG_BYTES = 32;

    private Hashes() {
    }

    /**
     * Unkeyed SHA-256 over {@code data}, used for audit payload commitments.
     *
     * <p>Mirrors the C++ {@code sha256} and the plain {@code blake3::hash} of
     * the Rust audit module. This is standard FIPS 180-4 SHA-256, so the bytes
     * agree across the C++, Java, and any other standard implementation.
     *
     * @return the 32-byte digest
     */
    public static byte[] sha256(byte[] data) {
        Objects.requireNonNull(data, "data");
        return sha256Digest().digest(data);
    }

    /**
     * RFC 2104 HMAC-SHA-256 over the concatenation of {@code parts} under
     * {@code key}.
     *
     * <p>The concatenation is the message the tag is computed over, so
     * splitting a message into parts never changes the tag. Mirrors
     * {@code keyed_tag}: a 32-byte key, a 32-byte result; the key is shorter
     * than the 64-byte SHA-256 block, so HMAC zero-pads it to the block length
     * before mixing the inner and outer pads.
     *
     * @param key   the 32-byte key
     * @param parts the message parts, concatenated in order
     * @throws IllegalArgumentException when {@code key} is not 32 bytes long
     */
    public static byte[] keyedTag(byte[] key, byte[]... parts) {
        Objects.requireNonNull(key, "key");
        if (key.length != TAG_BYTES) {
            throw new IllegalArgumentException(
                    "keyedTag: key must be " + TAG_BYTES + " bytes, got " + key.length);
        }
        Objects.requireNonNull(parts, "parts");
        try {
            Mac mac = Mac.getInstance("HmacSHA256");
            mac.init(new SecretKeySpec(key, "HmacSHA256"));
            for (byte[] part : parts) {
                mac.update(Objects.requireNonNull(part, "part"));
            }
            return mac.doFinal();
        } catch (NoSuchAlgorithmException | InvalidKeyException e) {
            throw new IllegalStateException("HmacSHA256 is unavailable in this runtime", e);
        }
    }

    /**
     * Little-endian encoding of a 16-bit value, used by seals and receipts.
     *
     * @throws IllegalArgumentException when {@code value} is outside [0, 65535]
     */
    public static byte[] le16(int value) {
        if (value < 0 || value > 0xFFFF) {
            throw new IllegalArgumentException("le16: value outside [0, 65535]: " + value);
        }
        return new byte[] {(byte) (value & 0xFF), (byte) ((value >>> 8) & 0xFF)};
    }

    /**
     * Little-endian encoding of the low 64 bits of {@code value}, used by
     * seals and receipts. The C++ parameter is {@code uint64_t}; the Java
     * {@code long} of the same numeric value encodes to the same bytes.
     */
    public static byte[] le64(long value) {
        byte[] out = new byte[8];
        for (int i = 0; i < out.length; i++) {
            out[i] = (byte) (value >>> (8 * i));
        }
        return out;
    }

    /**
     * Converts a 32-character ASCII key literal into its bytes. Mirrors the
     * C++ {@code make_key}, which accepts exactly a 32-byte key literal.
     *
     * @throws IllegalArgumentException when the literal is not 32 ASCII characters
     */
    static byte[] asciiKey(String text) {
        Objects.requireNonNull(text, "text");
        if (text.length() != TAG_BYTES) {
            throw new IllegalArgumentException(
                    "key literal must hold exactly " + TAG_BYTES + " characters, got " + text.length());
        }
        byte[] key = new byte[TAG_BYTES];
        for (int i = 0; i < TAG_BYTES; i++) {
            char c = text.charAt(i);
            if (c > 0x7F) {
                throw new IllegalArgumentException(
                        "key literal must be ASCII, found U+" + Integer.toHexString(c));
            }
            key[i] = (byte) c;
        }
        return key;
    }

    /** A fresh SHA-256 digest; the algorithm is mandatory on every JDK. */
    private static MessageDigest sha256Digest() {
        try {
            return MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 is unavailable in this runtime", e);
        }
    }
}
