#pragma once

// Shared types for the tagma-sec layer. Mirrors sw/rust/sec/src/types.rs.

#include "tagma_core/coord.h"
#include "tagma_sec/hash.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace tagma_sec {

// Identity of a registered principal.
using PrincipalId = uint64_t;

// Monotonic counter that scopes replay protection.
using Epoch = uint64_t;

// Action label. The PoC workflow uses a single action (route update).
using Action = uint8_t;

// The single action of the PoC workflow: a coordination node route update.
constexpr Action ACTION_ROUTE_UPDATE = 1;

// A scoped path: an ordered sequence of valid Coords.
using Path = std::vector<tagma::Coord>;

// Authorization decision.
enum class Decision { Allow, Deny };

// A scope names a target path or a set of paths.
class Scope {
 public:
  // Exact match: the target path equals the scope path.
  static Scope exact(Path p) { return Scope(Kind::Exact, std::move(p)); }
  // Prefix match: the scope path is a prefix of the target path.
  static Scope prefix(Path p) { return Scope(Kind::Prefix, std::move(p)); }

  // Returns true when `path` falls inside this scope under the Milestone 1
  // matching rules (Exact and Prefix only).
  bool matches(const Path& path) const {
    if (kind_ == Kind::Exact) return path_ == path;
    if (path_.size() > path.size()) return false;
    return std::equal(path_.begin(), path_.end(), path.begin());
  }

  // A stable key for revocation bookkeeping: a rule marker byte followed by
  // the coordinate indices of the scope path.
  std::vector<uint16_t> key() const {
    std::vector<uint16_t> k;
    k.push_back(kind_ == Kind::Exact ? 0 : 1);
    for (const auto& c : path_) k.push_back(c.index());
    return k;
  }

  bool operator==(const Scope& other) const {
    return kind_ == other.kind_ && path_ == other.path_;
  }
  bool operator!=(const Scope& other) const { return !(*this == other); }

  const Path& path() const { return path_; }

 private:
  enum class Kind { Exact, Prefix };
  Scope(Kind kind, Path path) : kind_(kind), path_(std::move(path)) {}
  Kind kind_;
  Path path_;
};

// A capability statement binding a principal to a scope, with a lifetime.
struct Attestation {
  uint64_t id;
  PrincipalId principal;
  Scope scope;
  Epoch issued_epoch;
  Epoch valid_until;
};

// Integrity binding over a record, its path, a principal, and an epoch.
struct Seal {
  std::array<uint8_t, 32> tag;
};

// An entry in the append-only audit log.
struct Event {
  uint64_t id;
  std::optional<uint64_t> prev;
  std::array<uint8_t, 32> payload_hash;
  Epoch epoch;
};

// Signed evidence for non-repudiation of origin.
struct SignedEvidence {
  Bytes evidence;
  std::array<uint8_t, 32> tag;
};

// Proof of delivery and origin for exchanged evidence.
struct Receipt {
  SignedEvidence signed_evidence;
  PrincipalId remote;
  Epoch epoch;
};

}  // namespace tagma_sec
