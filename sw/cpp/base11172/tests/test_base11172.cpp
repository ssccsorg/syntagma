// Verifies the base11172 serialization: u16 round trip, byte round trip,
// binary round trip, odd-byte padding, and invalid input rejection. The
// checks mirror the Rust base11172 crate tests.

#include <base11172/base11172.h>

#include <cstdio>
#include <cstdint>
#include <string>
#include <vector>

namespace {

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

void test_roundtrip_u16() {
  const uint16_t values[] = {0, 1, 11171, 12345, 32768, 65535};
  for (const uint16_t value : values) {
    const auto [c0, c1] = base11172::encode_u16(value);
    const auto decoded = base11172::decode_pair(c0, c1);
    check(decoded.has_value() && *decoded == value, "u16 round trip");
  }
}

void test_roundtrip_bytes() {
  // "Hello, Base11172!" is 17 bytes (odd), so the decoded vector carries
  // one zero padding byte; the reference slices [..data.len()].
  const std::string data = "Hello, Base11172!";
  const std::u32string encoded =
      base11172::encode_bytes(reinterpret_cast<const uint8_t*>(data.data()),
                              data.size());
  const auto decoded = base11172::decode_bytes(encoded);
  check(decoded.has_value() && decoded->size() >= data.size(), "bytes decode");
  if (decoded && decoded->size() >= data.size()) {
    check(std::equal(decoded->begin(), decoded->begin() + data.size(),
                     data.begin()),
          "bytes round trip");
  }
}

void test_binary_roundtrip() {
  std::vector<uint8_t> data(256);
  for (int i = 0; i < 256; ++i) data[i] = static_cast<uint8_t>(i);
  const std::u32string encoded = base11172::encode_bytes(data.data(), data.size());
  const auto decoded = base11172::decode_bytes(encoded);
  check(decoded.has_value() && *decoded == data, "binary round trip");
}

void test_odd_byte_padding() {
  const uint8_t single[] = {0x41};
  const std::u32string encoded = base11172::encode_bytes(single, 1);
  const auto decoded = base11172::decode_bytes(encoded);
  check(decoded.has_value() && decoded->size() == 2 && (*decoded)[0] == 0x41 &&
            (*decoded)[1] == 0x00,
        "odd byte padded with zero high byte");
}

void test_invalid_input() {
  check(!base11172::decode_pair(U'\0', U'가').has_value(),
        "null char rejected");
  check(!base11172::decode_pair(U'가', U'\0').has_value(),
        "null second char rejected");
  check(!base11172::decode_pair(U'ힰ', U'가').has_value(),
        "char beyond block rejected");
  check(!base11172::decode_bytes(U"가").has_value(), "odd count rejected");
  check(!base11172::decode_bytes(U"가ힰ").has_value(), "invalid char rejected");
}

}  // namespace

int main() {
  test_roundtrip_u16();
  test_roundtrip_bytes();
  test_binary_roundtrip();
  test_odd_byte_padding();
  test_invalid_input();

  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("base11172: all checks passed\n");
  return 0;
}
