#pragma once

// tagma-sec pattern. Mirrors sw/rust/sec/src/delos.rs.
//
// The authority matches scopes over path sequences (Exact and Prefix) and
// keeps an epoch-scoped revocation list. The integrity seal binds the record,
// the path, the principal, and the epoch, so replay across epochs is
// detected.

#include "tagma_sec/authority.h"
#include "tagma_sec/integrity.h"

#include <map>
#include <utility>
#include <vector>

namespace tagma_sec {

// CoordPath-scoped authority with epoch-scoped revocation.
class DelosAuthority : public Authority {
 public:
  PrincipalId register_principal(const Bytes& cred) override;
  std::optional<Attestation> issue(PrincipalId principal, Scope scope,
                                   Epoch issued_epoch,
                                   Epoch valid_until) override;
  Decision authorize(const Attestation& att, const Path& path, Action action,
                     Epoch epoch) override;
  void revoke(PrincipalId principal, const Scope& scope, Epoch epoch) override;

 private:
  uint64_t next_id_ = 0;
  uint64_t next_att_ = 0;
  std::map<PrincipalId, std::vector<Attestation>> attestations_;
  std::map<std::pair<PrincipalId, std::vector<uint16_t>>, Epoch> revoked_;
};

// Integrity seal for the tagma-sec pattern: binds record, path, principal,
// and epoch.
class DelosIntegrity : public Integrity {
 public:
  Seal seal(const Bytes& record, const Path& path, PrincipalId principal,
            Epoch epoch) override;
  bool verify(const Bytes& record, const Path& path, PrincipalId principal,
              Epoch epoch, const Seal& seal) override;
  std::optional<Seal> refresh(const Bytes& record, const Path& path,
                              PrincipalId principal, Epoch from_epoch,
                              Epoch to_epoch, const Seal& seal) override;
};

}  // namespace tagma_sec
