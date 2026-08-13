#pragma once

// Audit: append-only chained evidence log. Mirrors sw/rust/sec/src/audit.rs.
//
// Every entry chains to its predecessor, so retroactive modification of an
// interior entry breaks the chain. Inclusion proofs and evidence bundles pair
// each event with its payload so an external party can verify commitments
// without access to the log.

#include "tagma_sec/types.h"

#include <cstdint>
#include <optional>
#include <vector>

namespace tagma_sec {

// An event paired with the payload it commits to.
struct Entry {
  Event event;
  Bytes payload;
};

// Chain segment from genesis through a target entry, verifiable without the
// log.
class InclusionProof {
 public:
  std::vector<Entry> entries;

  // Verifies ids 0..=n, prev links, and payload commitments.
  bool verify() const;
};

// Exportable evidence bundle over a range of entries.
class EvidenceBundle {
 public:
  std::vector<Entry> entries;

  // Verifies internal chain consistency and payload commitments.
  bool verify() const;
};

class Audit {
 public:
  virtual ~Audit() = default;

  // Appends an event for `payload` at `epoch` and returns the entry.
  virtual Event append(const Bytes& payload, Epoch epoch) = 0;

  // Returns true when the chain is intact over entries `from..=to`.
  virtual bool verify_chain(uint64_t from, uint64_t to) = 0;

  // Returns an inclusion proof for `entry`, or nullopt when out of range.
  virtual std::optional<InclusionProof> prove(uint64_t entry) = 0;

  // Returns an exportable evidence bundle over entries `from..=to`.
  virtual std::optional<EvidenceBundle> export_range(uint64_t from,
                                                     uint64_t to) = 0;
};

// Chained append-only log shared by the legacy and tagma-sec stacks.
class ChainedAudit : public Audit {
 public:
  Event append(const Bytes& payload, Epoch epoch) override;
  bool verify_chain(uint64_t from, uint64_t to) override;
  std::optional<InclusionProof> prove(uint64_t entry) override;
  std::optional<EvidenceBundle> export_range(uint64_t from, uint64_t to) override;

 private:
  std::vector<Event> events_;
  std::vector<Bytes> payloads_;
};

}  // namespace tagma_sec
