// Hash-layer tests for the tagma-sec module: the SHA-256 payload commitment
// and the RFC 2104 keyed tag, each pinned against an independently computed
// vector.
//
// The module's public keyed entry point takes exactly a 32-byte key, so the
// RFC 4231 test shape is used with the key length adapted to the interface:
// key = thirty-two 0x0b bytes, message = "Hi There". The construction is
// unchanged RFC 2104 HMAC-SHA-256, and HMAC zero-pads keys shorter than the
// 64-byte block, so the digest below is what any standard implementation
// produces for that key and message. It was computed with OpenSSL
// (`printf 'Hi There' | openssl dgst -sha256 -mac HMAC
// -macopt hexkey:0b0b...0b`, 32 key bytes), independently of this code.

#include <tagma_sec/hash.h>

#include <array>
#include <cstdio>
#include <string>

namespace {

using namespace tagma_sec;

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

Bytes bytes(const std::string& s) { return Bytes(s.begin(), s.end()); }

std::string hex(const std::array<uint8_t, 32>& tag) {
  static const char* digits = "0123456789abcdef";
  std::string out;
  out.reserve(tag.size() * 2);
  for (uint8_t b : tag) {
    out.push_back(digits[b >> 4]);
    out.push_back(digits[b & 0x0f]);
  }
  return out;
}

// The module key shape: thirty-two 0x0b bytes.
std::array<uint8_t, 32> key_0b() {
  std::array<uint8_t, 32> key{};
  key.fill(0x0b);
  return key;
}

void test_sha256_matches_the_fips_180_4_vectors() {
  // FIPS 180-4: SHA-256("abc"), one short block.
  check(hex(sha256(bytes("abc"))) ==
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "SHA-256 of abc matches the FIPS 180-4 vector");

  // FIPS 180-4: the 56-byte message, whose padding needs a second block.
  check(hex(sha256(bytes("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"))) ==
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        "SHA-256 of the 56-byte FIPS message matches");

  // FIPS 180-4: the 112-byte message, two full blocks plus padding.
  check(hex(sha256(bytes("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn"
                         "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"))) ==
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
        "SHA-256 of the 112-byte FIPS message matches");
}

void test_keyed_tag_matches_the_openssl_vector() {
  check(hex(keyed_tag(key_0b(), {bytes("Hi There")})) ==
            "198a607eb44bfbc69903a0f1cf2bbdc5ba0aa3f3d9ae3c1c7a3b1696a0b68cf7",
        "keyed tag matches the openssl-derived HMAC-SHA-256 vector");
}

void test_keyed_tag_concatenates_parts() {
  check(keyed_tag(key_0b(), {bytes("Hi "), bytes("There")}) ==
            keyed_tag(key_0b(), {bytes("Hi There")}),
        "splitting the message into parts does not change the tag");
}

}  // namespace

int main() {
  test_sha256_matches_the_fips_180_4_vectors();
  test_keyed_tag_matches_the_openssl_vector();
  test_keyed_tag_concatenates_parts();

  if (failures == 0) {
    std::printf("test_hash: 5 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_hash: %d failures\n", failures);
  return 1;
}
