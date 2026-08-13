// Legacy security pattern. Mirrors sw/rust/sec/src/legacy.rs.

#include "tagma_sec/legacy.h"

#include "tagma_sec/hash.h"

#include <algorithm>

namespace tagma_sec {
namespace {

constexpr std::array<uint8_t, 32> make_key(const char (&s)[33]) {
  std::array<uint8_t, 32> k{};
  for (int i = 0; i < 32; ++i) k[i] = static_cast<uint8_t>(s[i]);
  return k;
}

const auto kLegacyKey = make_key("tagma-sec-poc-legacy-int-key-000");

}  // namespace

PrincipalId LegacyAuthority::register_principal(const Bytes&) { return ++next_id_; }

std::optional<Attestation> LegacyAuthority::issue(PrincipalId principal, Scope scope,
                                                  Epoch issued_epoch,
                                                  Epoch valid_until) {
  Attestation att{++next_att_, principal, std::move(scope), issued_epoch, valid_until};
  attestations_[principal].push_back(att);
  return att;
}

Decision LegacyAuthority::authorize(const Attestation& att, const Path& path, Action,
                                    Epoch epoch) {
  const auto it = attestations_.find(att.principal);
  if (it == attestations_.end()) return Decision::Deny;
  for (const auto& a : it->second) {
    if (a.id == att.id && a.scope.matches(path) && epoch >= a.issued_epoch &&
        epoch <= a.valid_until) {
      return Decision::Allow;
    }
  }
  return Decision::Deny;
}

void LegacyAuthority::revoke(PrincipalId principal, const Scope& scope, Epoch) {
  const auto it = attestations_.find(principal);
  if (it == attestations_.end()) return;
  it->second.erase(
      std::remove_if(it->second.begin(), it->second.end(),
                     [&scope](const Attestation& a) { return a.scope == scope; }),
      it->second.end());
}

Seal LegacyIntegrity::seal(const Bytes& record, const Path& path, PrincipalId,
                           Epoch) {
  std::vector<Bytes> parts;
  parts.push_back(record);
  for (const auto& c : path) parts.push_back(le16(c.index()));
  return Seal{keyed_tag(kLegacyKey, parts)};
}

bool LegacyIntegrity::verify(const Bytes& record, const Path& path, PrincipalId,
                             Epoch, const Seal& seal) {
  return this->seal(record, path, 0, 0).tag == seal.tag;
}

std::optional<Seal> LegacyIntegrity::refresh(const Bytes& record, const Path& path,
                                             PrincipalId principal, Epoch,
                                             Epoch to_epoch, const Seal& seal) {
  if (!this->verify(record, path, principal, 0, seal)) return std::nullopt;
  return this->seal(record, path, principal, to_epoch);
}

}  // namespace tagma_sec
