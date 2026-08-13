#pragma once

// Proxy: composes the four modules behind polymorphic interfaces and
// switches between the legacy and tagma-sec stacks. Mirrors
// sw/rust/sec/src/proxy.rs. The scenario workflow is written only against
// the interfaces, so the same requirements and outputs are exercised against
// both implementations.

#include "tagma_sec/audit.h"
#include "tagma_sec/authority.h"
#include "tagma_sec/channel.h"
#include "tagma_sec/integrity.h"

#include <memory>
#include <optional>

namespace tagma_sec {

// Outcome of a successful route update: the record's audit entry and the
// receipt proving delivery and origin (non-repudiation).
struct RouteUpdate {
  Event event;
  Receipt receipt;
};

// A composable security stack with switchable implementations.
class SecStack {
 public:
  // The legacy security pattern stack.
  static SecStack legacy();
  // The tagma-sec pattern stack.
  static SecStack delos();

  std::unique_ptr<Authority> authority;
  std::unique_ptr<Integrity> integrity;
  std::unique_ptr<Audit> audit;
  std::unique_ptr<Channel> channel;
};

// Scenario workflow: a coordination node submits a route update.
//
// The update is accepted only when the attestation authorizes the path and
// action at the current epoch and the sealed record verifies. The record is
// then committed to the audit log, and the channel exchanges a receipt
// binding the record to its path, epoch, and origin principal; the signed
// evidence is committed to the audit log as the non-repudiation receipt.
// Returns nullopt on any rejected step.
std::optional<RouteUpdate> route_update(SecStack& stack, const Attestation& att,
                                        const Path& path, const Bytes& record,
                                        Epoch epoch);

}  // namespace tagma_sec
