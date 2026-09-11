// Module-level tag and commitment vectors pinned against independent OpenSSL
// computations.
//
// Each value below was computed with OpenSSL from the layout the module
// documents (key literal, part order, little-endian encodings), not from this
// code. A drift in a key literal, a part order, or a length encoding therefore
// fails here and in the Java mirror instead of passing silently: the shared
// workflow tests only recompute and compare, so they cannot see such a change.

#include <tagma_sec/audit.h>
#include <tagma_sec/channel.h>
#include <tagma_sec/delos.h>
#include <tagma_sec/legacy.h>
#include <tagma_sec/proxy.h>

#include <array>
#include <cstdint>
#include <cstdio>
#include <string>
#include <vector>

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

Path path(const std::vector<uint16_t>& idxs) {
  Path p;
  for (uint16_t i : idxs) p.push_back(*tagma::Coord::from_index(i));
  return p;
}

std::string to_hex(const std::array<uint8_t, 32>& tag) {
  static const char* digits = "0123456789abcdef";
  std::string out;
  out.reserve(tag.size() * 2);
  for (uint8_t b : tag) {
    out.push_back(digits[b >> 4]);
    out.push_back(digits[b & 0x0f]);
  }
  return out;
}

void test_legacy_seal_tag_is_pinned() {
  LegacyIntegrity integrity;
  const Seal seal = integrity.seal(bytes("route-v1"), path({1, 2}), 7, 5);
  check(to_hex(seal.tag) ==
            "3ea058c754d3e7a619ebc94ea9e08bdfb24c69349c1a71c8a5e453b6516ccc0f",
        "legacy seal tag matches the openssl vector");
}

void test_delos_seal_tag_is_pinned() {
  DelosIntegrity integrity;
  const Seal seal = integrity.seal(bytes("route-v1"), path({1, 2}), 7, 5);
  check(to_hex(seal.tag) ==
            "fd70c7dd4ef7652099a8feef0e86d62ddd572fdcf545e32ce6ae2caede0a56f9",
        "delos seal tag matches the openssl vector");
}

void test_channel_tag_is_pinned() {
  MacChannel channel;
  const SignedEvidence signed_evidence = channel.sign(bytes("route-update-request"));
  check(to_hex(signed_evidence.tag) ==
            "d9242ea0622659adea42adc0e7a1cdbe70355c4ca7bff008ddd33f8ebad8c7b4",
        "channel tag matches the openssl vector");
}

void test_audit_commitment_is_pinned() {
  ChainedAudit audit;
  const Event event = audit.append(bytes("event-0"), 0);
  check(to_hex(event.payload_hash) ==
            "23352a67ac7ffc4b5d98023dc1a54ee507d4bf95dc1c4154045c5e2bbde73317",
        "audit commitment matches the openssl vector");
}

void test_workflow_receipt_tag_is_pinned() {
  SecStack stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 0, 100);
  check(att.has_value(), "issuance succeeds");
  const auto res = route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 50);
  check(res.has_value(), "update accepted");
  if (res) {
    // The receipt tag is the channel tag over the concatenation route_update
    // exchanges: record | le16 path | le64 epoch | le64 remote | le64 epoch.
    check(to_hex(res->receipt.signed_evidence.tag) ==
              "ce8ef6f248b2426f1900821041662ef4d42821e1e77c0ad6479d0cec07f319c5",
          "workflow receipt tag matches the openssl vector");
    check(res->receipt.remote == principal && res->receipt.epoch == 50,
          "the pinned receipt binds the first registered principal at epoch 50");
  }
}

}  // namespace

int main() {
  test_legacy_seal_tag_is_pinned();
  test_delos_seal_tag_is_pinned();
  test_channel_tag_is_pinned();
  test_audit_commitment_is_pinned();
  test_workflow_receipt_tag_is_pinned();

  if (failures == 0) {
    std::printf("test_vectors: 6 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_vectors: %d failures\n", failures);
  return 1;
}
