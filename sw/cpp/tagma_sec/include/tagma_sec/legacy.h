#pragma once

// Legacy security pattern. Mirrors sw/rust/sec/src/legacy.rs.
//
// The authority is an ACL-style registry of attestations per principal.
// Revocation removes the scope immediately. The integrity seal binds only
// the record and the path; the principal and the epoch do not participate.
// This is the baseline that the tagma-sec pattern is compared against in
// the shared workflow tests.

#include "tagma_sec/authority.h"
#include "tagma_sec/integrity.h"

#include <map>
#include <vector>

namespace tagma_sec {

// ACL-style authority for the legacy pattern.
class LegacyAuthority : public Authority {
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
};

// Integrity seal for the legacy pattern: binds record and path only.
class LegacyIntegrity : public Integrity {
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
