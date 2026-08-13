// Chained audit log, inclusion proofs, and evidence bundles. Mirrors
// sw/rust/sec/src/audit.rs.

#include "tagma_sec/audit.h"

#include "tagma_sec/hash.h"

namespace tagma_sec {
namespace {

// Shared verification: payload commitments and prev links. The first entry
// may carry a prev link to an entry outside the bundle, so internal
// consistency is checked from the second entry onward.
bool verify_entries(const std::vector<Entry>& entries) {
  for (size_t i = 0; i < entries.size(); ++i) {
    if (sha256(entries[i].payload) != entries[i].event.payload_hash) return false;
    if (i > 0) {
      if (!entries[i].event.prev.has_value()) return false;
      if (entries[i].event.prev.value() != entries[i - 1].event.id) return false;
    }
  }
  return true;
}

}  // namespace

bool InclusionProof::verify() const {
  if (entries.empty()) return false;
  if (entries.front().event.prev.has_value()) return false;
  for (size_t i = 0; i < entries.size(); ++i) {
    if (entries[i].event.id != static_cast<uint64_t>(i)) return false;
  }
  return verify_entries(entries);
}

bool EvidenceBundle::verify() const { return verify_entries(entries); }

Event ChainedAudit::append(const Bytes& payload, Epoch epoch) {
  std::optional<uint64_t> prev;
  if (!events_.empty()) prev = events_.back().id;
  const uint64_t id = static_cast<uint64_t>(events_.size());
  Event event{id, prev, sha256(payload), epoch};
  events_.push_back(event);
  payloads_.push_back(payload);
  return event;
}

bool ChainedAudit::verify_chain(uint64_t from, uint64_t to) {
  if (from > to || to >= events_.size()) return false;
  for (uint64_t i = from; i <= to; ++i) {
    if (i == 0) {
      if (events_[i].prev.has_value()) return false;
    } else if (events_[i].prev != events_[i - 1].id) {
      return false;
    }
  }
  return true;
}

std::optional<InclusionProof> ChainedAudit::prove(uint64_t entry) {
  if (entry >= events_.size()) return std::nullopt;
  InclusionProof proof;
  for (uint64_t i = 0; i <= entry; ++i) {
    proof.entries.push_back(Entry{events_[i], payloads_[i]});
  }
  return proof;
}

std::optional<EvidenceBundle> ChainedAudit::export_range(uint64_t from, uint64_t to) {
  if (from > to || to >= events_.size()) return std::nullopt;
  EvidenceBundle bundle;
  for (uint64_t i = from; i <= to; ++i) {
    bundle.entries.push_back(Entry{events_[i], payloads_[i]});
  }
  return bundle;
}

}  // namespace tagma_sec
