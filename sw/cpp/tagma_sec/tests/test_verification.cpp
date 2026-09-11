// Rejection paths of the verification branches: inclusion proofs, evidence
// bundles, and receipts.
//
// The happy paths are covered by the workflow and scenario suites; these
// cases pin the negative branches their javadoc describes, so deleting the
// genesis check, the id check, the prev-link check, or the truncated-evidence
// guard fails here instead of passing silently.

#include <tagma_sec/audit.h>
#include <tagma_sec/channel.h>
#include <tagma_sec/hash.h>
#include <tagma_sec/types.h>

#include <array>
#include <cstdint>
#include <cstdio>
#include <optional>
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

// An entry whose commitment matches its payload, with the id also used as the
// epoch so every other check of the verification branches still passes.
Entry entry(uint64_t id, std::optional<uint64_t> prev, const std::string& payload) {
  const Bytes body = bytes(payload);
  return Entry{Event{id, prev, sha256(body), id}, body};
}

void test_inclusion_proof_rejects_a_non_genesis_first_entry() {
  const InclusionProof proof{{entry(0, 0, "event-0")}};
  check(!proof.verify(), "a first entry carrying a prev link is rejected");
}

void test_inclusion_proof_rejects_shifted_ids() {
  const InclusionProof proof{{entry(1, std::nullopt, "event-0"),
                             entry(2, 1, "event-1")}};
  check(!proof.verify(), "ids that do not start at zero are rejected");
}

void test_inclusion_proof_rejects_an_empty_segment() {
  const InclusionProof proof{{}};
  check(!proof.verify(), "an empty segment is rejected");
}

void test_evidence_bundle_rejects_broken_prev_links() {
  const EvidenceBundle unrelated{{entry(0, std::nullopt, "event-0"),
                                 entry(1, 5, "event-1")}};
  check(!unrelated.verify(), "a prev link to an unrelated id is rejected");

  const EvidenceBundle missing{{entry(0, std::nullopt, "event-0"),
                                entry(1, std::nullopt, "event-1")}};
  check(!missing.verify(), "a missing prev link after the first entry is rejected");
}

void test_receipt_evidence_length_boundary() {
  MacChannel channel;
  const Bytes remote = le64(7);
  const Bytes epoch = le64(5);
  Bytes body;
  body.insert(body.end(), remote.begin(), remote.end());
  body.insert(body.end(), epoch.begin(), epoch.end());

  const Receipt exact{channel.sign(body), 7, 5};
  check(channel.verify_receipt(exact), "a 16-byte receipt body verifies");

  const Bytes truncated(body.begin(), body.end() - 1);
  const Receipt short_evidence{channel.sign(truncated), 7, 5};
  check(!channel.verify_receipt(short_evidence),
        "a receipt with truncated evidence is rejected");
}

}  // namespace

int main() {
  test_inclusion_proof_rejects_a_non_genesis_first_entry();
  test_inclusion_proof_rejects_shifted_ids();
  test_inclusion_proof_rejects_an_empty_segment();
  test_evidence_bundle_rejects_broken_prev_links();
  test_receipt_evidence_length_boundary();

  if (failures == 0) {
    std::printf("test_verification: 7 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_verification: %d failures\n", failures);
  return 1;
}
