// Proxy composition and the route-update workflow. Mirrors
// sw/rust/sec/src/proxy.rs.

#include "tagma_sec/proxy.h"

#include "tagma_sec/delos.h"
#include "tagma_sec/hash.h"
#include "tagma_sec/legacy.h"

#include <memory>

namespace tagma_sec {

SecStack SecStack::legacy() {
  SecStack s;
  s.authority = std::make_unique<LegacyAuthority>();
  s.integrity = std::make_unique<LegacyIntegrity>();
  s.audit = std::make_unique<ChainedAudit>();
  s.channel = std::make_unique<MacChannel>();
  return s;
}

SecStack SecStack::delos() {
  SecStack s;
  s.authority = std::make_unique<DelosAuthority>();
  s.integrity = std::make_unique<DelosIntegrity>();
  s.audit = std::make_unique<ChainedAudit>();
  s.channel = std::make_unique<MacChannel>();
  return s;
}

std::optional<RouteUpdate> route_update(SecStack& stack, const Attestation& att,
                                        const Path& path, const Bytes& record,
                                        Epoch epoch) {
  if (stack.authority->authorize(att, path, ACTION_ROUTE_UPDATE, epoch) !=
      Decision::Allow) {
    return std::nullopt;
  }
  const Seal seal = stack.integrity->seal(record, path, att.principal, epoch);
  if (!stack.integrity->verify(record, path, att.principal, epoch, seal)) {
    return std::nullopt;
  }
  const Event event = stack.audit->append(record, epoch);

  // Non-repudiation: bind the record to its path and epoch, exchange a
  // receipt with the channel, and commit the signed evidence to the audit
  // log (receipts are appended by the caller per docs/spec/tagma-sec.md).
  Bytes evidence;
  evidence.reserve(record.size() + path.size() * 2 + 8);
  evidence.insert(evidence.end(), record.begin(), record.end());
  for (const auto& c : path) {
    const Bytes idx = le16(c.index());
    evidence.insert(evidence.end(), idx.begin(), idx.end());
  }
  const Bytes epoch_bytes = le64(epoch);
  evidence.insert(evidence.end(), epoch_bytes.begin(), epoch_bytes.end());

  const Receipt receipt = stack.channel->exchange(evidence, att.principal, epoch);
  stack.audit->append(receipt.signed_evidence.evidence, epoch);

  return RouteUpdate{event, receipt};
}

}  // namespace tagma_sec
