#pragma once

// Authority: principal registration, attestation issuance, authorization,
// and revocation. Mirrors sw/rust/sec/src/authority.rs.
//
// Scope matching follows the Milestone 1 contract: Exact and Prefix rules
// only. Authorization never depends on path secrecy; paths are observable
// and replayable by design.

#include "tagma_sec/types.h"

#include <optional>

namespace tagma_sec {

class Authority {
 public:
  virtual ~Authority() = default;

  // Registers a principal from its credentials and returns its identity.
  virtual PrincipalId register_principal(const Bytes& cred) = 0;

  // Issues an attestation scoped to `scope`, valid from `issued_epoch`
  // through `valid_until`.
  virtual std::optional<Attestation> issue(PrincipalId principal, Scope scope,
                                           Epoch issued_epoch,
                                           Epoch valid_until) = 0;

  // Decides whether `att` authorizes `path` for `action` at `epoch`.
  // The action is carried by the interface; the Milestone 1 policy is
  // scope-only, so implementations do not yet discriminate by action.
  virtual Decision authorize(const Attestation& att, const Path& path,
                             Action action, Epoch epoch) = 0;

  // Revokes the given scope for the principal, effective from `epoch`.
  virtual void revoke(PrincipalId principal, const Scope& scope, Epoch epoch) = 0;
};

}  // namespace tagma_sec
