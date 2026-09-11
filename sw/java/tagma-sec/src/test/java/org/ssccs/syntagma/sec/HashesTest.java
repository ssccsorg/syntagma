package org.ssccs.syntagma.sec;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.ssccs.syntagma.sec.Fixtures.hex;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import org.junit.jupiter.api.Test;

/**
 * Byte-identity pins for the hash layer, the deliberate porting difference.
 *
 * <p>The Rust crate uses blake3; the C++ port implements SHA-256 (FIPS 180-4)
 * for payload commitments and RFC 2104 HMAC-SHA-256 for keyed tags, and the
 * Java port delegates both to the JDK. The SHA-256 vectors are the published
 * FIPS 180-4 examples; the keyed vector is the RFC 4231 shape adapted to the
 * module's fixed key length and was computed with OpenSSL (an implementation
 * independent of this code), so a failure means the tag no longer matches
 * standard HMAC-SHA-256.
 *
 * <p>The module's public keyed entry point takes exactly a 32-byte key, so the
 * key length is adapted while the construction stays RFC 2104: HMAC zero-pads
 * a key shorter than the 64-byte block, so the 4-byte RFC 4231 key "Jefe"
 * zero-extended to 32 bytes produces the published RFC 4231 test case 2
 * digest, which the second keyed vector below pins.
 */
class HashesTest {

    private static byte[] ascii(String text) {
        return text.getBytes(StandardCharsets.US_ASCII);
    }

    @Test
    void sha256MatchesFips180Vectors() {
        assertArrayEquals(hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
                Hashes.sha256(new byte[0]), "SHA-256 of the empty message");
        assertArrayEquals(hex("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
                Hashes.sha256(ascii("abc")), "SHA-256 of abc");
        assertArrayEquals(hex("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"),
                Hashes.sha256(ascii("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
                "SHA-256 of the 56-byte padding vector");
        assertArrayEquals(hex("cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"),
                Hashes.sha256(ascii("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn"
                        + "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu")),
                "SHA-256 of the 112-byte two-block vector");
        assertArrayEquals(hex("3e1acf18e9b268eb42f54230cd96c449d2bb6248a13e1d3f55322e7bfd66985e"),
                Hashes.sha256(ascii("route-v1")),
                "SHA-256 of the workflow record, as the C++ audit commits it");
    }

    @Test
    void keyedTagMatchesOpensslVector() {
        // Key = thirty-two 0x0b bytes, message = "Hi There", computed with:
        //   printf 'Hi There' | openssl dgst -sha256 -mac HMAC -macopt hexkey:0b0b...0b
        byte[] key = new byte[32];
        Arrays.fill(key, (byte) 0x0B);
        assertArrayEquals(hex("198a607eb44bfbc69903a0f1cf2bbdc5ba0aa3f3d9ae3c1c7a3b1696a0b68cf7"),
                Hashes.keyedTag(key, ascii("Hi There")),
                "keyed tag matches the OpenSSL HMAC-SHA-256 vector");
    }

    @Test
    void keyedTagMatchesRfc4231TestVectorThroughMessageAssembly() {
        // RFC 4231 test case 2: key "Jefe", message "what do ya want for
        // nothing?". HMAC zero-pads keys shorter than the block, so the 32-byte
        // zero-extension of the 4-byte key yields the published digest. The
        // message is passed in parts so the assembly path is exercised.
        byte[] jefeKey = new byte[32];
        System.arraycopy(ascii("Jefe"), 0, jefeKey, 0, 4);
        assertArrayEquals(hex("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
                Hashes.keyedTag(jefeKey, ascii("what do ya "), ascii("want for nothing?")),
                "RFC 4231 test case 2 through the part assembly");
        assertArrayEquals(hex("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
                Hashes.keyedTag(jefeKey, ascii("what do ya want for nothing?")),
                "RFC 4231 test case 2 as a single part");
    }

    @Test
    void keyedTagConcatenatesParts() {
        byte[] key = Hashes.asciiKey("tagma-sec-poc-delos-int-key-0000");
        assertArrayEquals(Hashes.keyedTag(key, ascii("ab")),
                Hashes.keyedTag(key, ascii("a"), ascii("b")),
                "splitting the message into parts does not change the tag");
        assertArrayEquals(Hashes.keyedTag(key, ascii("ab")),
                Hashes.keyedTag(key, ascii("a"), new byte[0], ascii("b")),
                "empty parts do not change the tag");
    }

    @Test
    void littleEndianEncodings() {
        assertArrayEquals(new byte[] {0x34, 0x12}, Hashes.le16(0x1234), "le16 of 0x1234");
        assertArrayEquals(new byte[] {0x00, 0x00}, Hashes.le16(0), "le16 of zero");
        assertArrayEquals(new byte[] {(byte) 0xFF, (byte) 0xFF}, Hashes.le16(0xFFFF),
                "le16 of 65535");
        assertArrayEquals(new byte[] {0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
                Hashes.le64(1), "le64 of one");
        assertArrayEquals(new byte[] {0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01},
                Hashes.le64(0x0102030405060708L), "le64 of 0x0102030405060708");
    }

    @Test
    void hashHelpersRejectOutOfDomainInputs() {
        assertThrows(IllegalArgumentException.class,
                () -> Hashes.keyedTag(new byte[31], new byte[0]), "31-byte key rejected");
        assertThrows(IllegalArgumentException.class,
                () -> Hashes.keyedTag(new byte[33], new byte[0]), "33-byte key rejected");
        assertThrows(IllegalArgumentException.class, () -> Hashes.le16(-1), "negative le16");
        assertThrows(IllegalArgumentException.class, () -> Hashes.le16(0x10000), "le16 overflow");
        assertThrows(IllegalArgumentException.class,
                () -> Hashes.asciiKey("tagma-sec-poc-legacy-int-key-00"), "short key literal");
    }
}
