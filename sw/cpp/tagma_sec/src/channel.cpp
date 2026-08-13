// Keyed-tag channel shared by the legacy and tagma-sec stacks. Mirrors
// sw/rust/sec/src/channel.rs.

#include "tagma_sec/channel.h"

#include "tagma_sec/hash.h"

namespace tagma_sec {
namespace {

constexpr std::array<uint8_t, 32> make_key(const char (&s)[33]) {
  std::array<uint8_t, 32> k{};
  for (int i = 0; i < 32; ++i) k[i] = static_cast<uint8_t>(s[i]);
  return k;
}

const auto kChannelKey = make_key("tagma-sec-poc-channel-key-000000");

}  // namespace

SignedEvidence MacChannel::sign(const Bytes& evidence) {
  return SignedEvidence{evidence, keyed_tag(kChannelKey, {evidence})};
}

bool MacChannel::verify(const SignedEvidence& signed_evidence) {
  return keyed_tag(kChannelKey, {signed_evidence.evidence}) == signed_evidence.tag;
}

Receipt MacChannel::exchange(const Bytes& local, PrincipalId remote, Epoch epoch) {
  Bytes evidence;
  evidence.reserve(local.size() + 16);
  evidence.insert(evidence.end(), local.begin(), local.end());
  const Bytes remote_bytes = le64(remote);
  const Bytes epoch_bytes = le64(epoch);
  evidence.insert(evidence.end(), remote_bytes.begin(), remote_bytes.end());
  evidence.insert(evidence.end(), epoch_bytes.begin(), epoch_bytes.end());
  return Receipt{sign(evidence), remote, epoch};
}

bool MacChannel::verify_receipt(const Receipt& receipt) {
  const Bytes& ev = receipt.signed_evidence.evidence;
  if (ev.size() < 16) return false;
  uint64_t remote = 0;
  uint64_t epoch = 0;
  for (int i = 0; i < 8; ++i) {
    remote |= static_cast<uint64_t>(ev[ev.size() - 16 + i]) << (8 * i);
    epoch |= static_cast<uint64_t>(ev[ev.size() - 8 + i]) << (8 * i);
  }
  return verify(receipt.signed_evidence) && remote == receipt.remote &&
         epoch == receipt.epoch;
}

}  // namespace tagma_sec
