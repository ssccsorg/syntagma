// tagma-sec pattern. Mirrors sw/rust/sec/src/delos.rs.

#include "tagma_sec/delos.h"

#include "tagma_sec/hash.h"

namespace tagma_sec {
namespace {

constexpr std::array<uint8_t, 32> make_key(const char (&s)[33]) {
  std::array<uint8_t, 32> k{};
  for (int i = 0; i < 32; ++i) k[i] = static_cast<uint8_t>(s[i]);
  return k;
}

const auto kDelosKey = make_key("tagma-sec-poc-delos-int-key-0000");

}  // namespace

PrincipalId DelosAuthority::register_principal(const Bytes&) { return ++next_id_; }

std::optional<Attestation> DelosAuthority::issue(PrincipalId principal, Scope scope,
                                                 Epoch issued_epoch,
                                                 Epoch valid_until) {
  Attestation att{++next_att_, principal, std::move(scope), issued_epoch, valid_until};
  attestations_[principal].push_back(att);
  return att;
}

Decision DelosAuthority::authorize(const Attestation& att, const Path& path, Action,
                                   Epoch epoch) {
  // The presented attestation carries the holder's identity claim; the
  // window, the scope, and the revocation record are read from the stored
  // attestation, so a presented copy cannot extend or reshape the grant.
  const auto it = attestations_.find(att.principal);
  if (it == attestations_.end()) return Decision::Deny;
  const Attestation* stored = nullptr;
  for (const auto& a : it->second) {
    if (a.id == att.id) {
      stored = &a;
      break;
    }
  }
  if (stored == nullptr) return Decision::Deny;
  if (!stored->scope.matches(path)) return Decision::Deny;
  if (epoch < stored->issued_epoch || epoch > stored->valid_until) {
    return Decision::Deny;
  }
  const auto r = revoked_.find({att.principal, stored->scope.key()});
  if (r != revoked_.end() && epoch >= r->second) return Decision::Deny;
  return Decision::Allow;
}

void DelosAuthority::revoke(PrincipalId principal, const Scope& scope, Epoch epoch) {
  revoked_[{principal, scope.key()}] = epoch;
}

Seal DelosIntegrity::seal(const Bytes& record, const Path& path,
                          PrincipalId principal, Epoch epoch) {
  std::vector<Bytes> parts;
  parts.push_back(record);
  parts.push_back(le64(principal));
  parts.push_back(le64(epoch));
  for (const auto& c : path) parts.push_back(le16(c.index()));
  return Seal{keyed_tag(kDelosKey, parts)};
}

bool DelosIntegrity::verify(const Bytes& record, const Path& path,
                            PrincipalId principal, Epoch epoch, const Seal& seal) {
  return this->seal(record, path, principal, epoch).tag == seal.tag;
}

std::optional<Seal> DelosIntegrity::refresh(const Bytes& record, const Path& path,
                                            PrincipalId principal, Epoch from_epoch,
                                            Epoch to_epoch, const Seal& seal) {
  if (to_epoch <= from_epoch) return std::nullopt;
  if (!this->verify(record, path, principal, from_epoch, seal)) return std::nullopt;
  return this->seal(record, path, principal, to_epoch);
}

}  // namespace tagma_sec
