// tagma-sec-specific properties of the Delos pattern stack. These tests pin
// the properties that distinguish the tagma-sec pattern from the legacy one:
// epoch-bound seals, prefix scope semantics, and revocation that is effective
// only from the recorded epoch. Mirrors sw/rust/sec/tests/delos.rs.

#include <tagma_sec/proxy.h>

#include <cstdio>
#include <string>
#include <vector>

namespace {

using namespace tagma_sec;

int failures = 0;

void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    failures += 1;
  }
}

Bytes bytes(const std::string& s) { return Bytes(s.begin(), s.end()); }

Path path(const std::vector<uint16_t>& idxs) {
  Path p;
  for (uint16_t i : idxs) p.push_back(*tagma::Coord::from_index(i));
  return p;
}

void test_delos_seal_binds_epoch_and_principal() {
  const Bytes record = bytes("route-v1");
  const Path target = path({1, 2});

  // Legacy seal binds only record and path: principal and epoch are free.
  auto legacy = SecStack::legacy();
  const Seal legacy_seal = legacy.integrity->seal(record, target, 7, 5);
  check(legacy.integrity->verify(record, target, 7, 5, legacy_seal), "legacy verifies");
  check(legacy.integrity->verify(record, target, 99, 99, legacy_seal),
        "legacy seal ignores principal and epoch");

  // Delos seal binds record, path, principal, and epoch.
  auto delos = SecStack::delos();
  const Seal delos_seal = delos.integrity->seal(record, target, 7, 5);
  check(delos.integrity->verify(record, target, 7, 5, delos_seal), "delos verifies");
  check(!delos.integrity->verify(record, target, 7, 6, delos_seal),
        "epoch change breaks the delos seal");
  check(!delos.integrity->verify(record, target, 8, 5, delos_seal),
        "principal change breaks the delos seal");
  check(!delos.integrity->verify(record, path({1, 3}), 7, 5, delos_seal),
        "path change breaks the delos seal");
}

void test_delos_refresh_rejects_non_newer_epoch() {
  auto delos = SecStack::delos();
  const Bytes record = bytes("route-v1");
  const Path target = path({1, 2});
  const Seal seal = delos.integrity->seal(record, target, 7, 5);

  check(!delos.integrity->refresh(record, target, 7, 5, 4, seal).has_value(),
        "refresh to an older epoch rejected");
  check(!delos.integrity->refresh(record, target, 7, 5, 5, seal).has_value(),
        "refresh to the same epoch rejected");
  check(delos.integrity->refresh(record, target, 7, 5, 6, seal).has_value(),
        "refresh to a newer epoch succeeds");
}

void test_delos_seal_changes_when_any_bound_input_changes() {
  auto delos = SecStack::delos();
  const Bytes record = bytes("route-v1");
  const Path target = path({1, 2});
  const Seal base = delos.integrity->seal(record, target, 7, 5);

  struct Case {
    Bytes record;
    Path path;
    PrincipalId principal;
    Epoch epoch;
  };
  const std::vector<Case> cases = {
      {bytes("route-v2"), path({1, 2}), 7, 5},  // record change
      {record, path({1, 3}), 7, 5},             // path change
      {record, path({1, 2}), 8, 5},             // principal change
      {record, path({1, 2}), 7, 6},             // epoch change
  };
  for (const auto& c : cases) {
    const Seal seal = delos.integrity->seal(c.record, c.path, c.principal, c.epoch);
    check(seal.tag != base.tag, "delos seal changes when any bound input changes");
  }
}

void test_refresh_epoch_binding_differs_from_legacy() {
  const Bytes record = bytes("route-v1");
  const Path target = path({1, 2});

  // Legacy refresh accepts any from_epoch and re-emits the identical tag.
  auto legacy = SecStack::legacy();
  const Seal legacy_seal = legacy.integrity->seal(record, target, 7, 5);
  const auto legacy_refreshed = legacy.integrity->refresh(record, target, 7, 99, 100, legacy_seal);
  check(legacy_refreshed.has_value() && legacy_refreshed->tag == legacy_seal.tag,
        "legacy seal is epoch-free, refresh is a no-op");
  check(legacy.integrity->refresh(record, target, 7, 5, 4, legacy_seal).has_value(),
        "legacy refresh stays epoch-free even toward an older epoch");

  // Delos refresh re-binds the seal to the new epoch.
  auto delos = SecStack::delos();
  const Seal delos_seal = delos.integrity->seal(record, target, 7, 5);
  const auto delos_refreshed = delos.integrity->refresh(record, target, 7, 5, 6, delos_seal);
  check(delos_refreshed.has_value(), "delos refresh from the binding epoch succeeds");
  if (delos_refreshed) {
    check(delos_refreshed->tag != delos_seal.tag, "delos refresh re-binds the epoch");
    check(delos.integrity->verify(record, target, 7, 6, *delos_refreshed),
          "refreshed seal verifies at the new epoch");
    check(!delos.integrity->verify(record, target, 7, 5, *delos_refreshed),
          "refreshed seal no longer verifies at the old epoch");
  }
  check(!delos.integrity->refresh(record, target, 7, 99, 6, delos_seal).has_value(),
        "refresh from a wrong epoch rejected");
}

void test_prefix_scope_covers_longer_paths() {
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 0, 100);
  check(att.has_value(), "issuance succeeds");
  check(stack.authority->authorize(*att, path({1, 2}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Allow,
        "prefix matches the scope path itself");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Allow,
        "prefix matches a longer path");
  check(stack.authority->authorize(*att, path({1, 2, 3, 4}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Allow,
        "prefix matches a much longer path");
  check(stack.authority->authorize(*att, path({1, 3}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Deny,
        "sibling path denied");
  check(stack.authority->authorize(*att, path({2, 3}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Deny,
        "unrelated path denied");
}

void test_exact_scope_matches_only_identical_path() {
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const auto att = stack.authority->issue(principal, Scope::exact(path({1, 2})), 0, 100);
  check(att.has_value(), "issuance succeeds");
  check(stack.authority->authorize(*att, path({1, 2}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Allow,
        "exact path allowed");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Deny,
        "longer path denied");
  check(stack.authority->authorize(*att, path({1, 3}), ACTION_ROUTE_UPDATE, 50) ==
            Decision::Deny,
        "different path denied");
}

void test_delos_revocation_is_effective_from_recorded_epoch() {
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const Scope scope = Scope::prefix(path({1, 2}));
  const auto att = stack.authority->issue(principal, scope, 0, 100);
  check(att.has_value(), "issuance succeeds");
  stack.authority->revoke(principal, Scope::prefix(path({1, 2})), 10);

  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 9) ==
            Decision::Allow,
        "revocation not effective before the recorded epoch");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 10) ==
            Decision::Deny,
        "revocation takes effect at the recorded epoch");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 11) ==
            Decision::Deny,
        "revocation persists after the recorded epoch");

  // Legacy contrast: the ACL entry is removed immediately.
  auto legacy = SecStack::legacy();
  const PrincipalId l_principal = legacy.authority->register_principal({});
  const auto l_att = legacy.authority->issue(l_principal, Scope::prefix(path({1, 2})), 0, 100);
  check(l_att.has_value(), "issuance succeeds");
  legacy.authority->revoke(l_principal, Scope::prefix(path({1, 2})), 10);
  check(legacy.authority->authorize(*l_att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 9) ==
            Decision::Deny,
        "legacy revocation removes the entry immediately");
}

void test_revocation_key_is_scope_specific() {
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const auto exact_att = stack.authority->issue(principal, Scope::exact(path({1, 2})), 0, 100);
  const auto prefix_att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 0, 100);
  check(exact_att.has_value() && prefix_att.has_value(), "issuance succeeds");

  stack.authority->revoke(principal, Scope::exact(path({1, 2})), 10);

  check(stack.authority->authorize(*exact_att, path({1, 2}), ACTION_ROUTE_UPDATE, 10) ==
            Decision::Deny,
        "exact scope is revoked");
  check(stack.authority->authorize(*prefix_att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 10) ==
            Decision::Allow,
        "prefix scope has a distinct revocation key");
}

void test_attestation_lifetime_boundary_is_inclusive() {
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 0, 10);
  check(att.has_value(), "issuance succeeds");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 10) ==
            Decision::Allow,
        "the validity end epoch is inclusive");
  check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 11) ==
            Decision::Deny,
        "the epoch after the window is denied");
}

void test_revoked_scope_stays_revoked_after_reissue() {
  // Delos: revocation is (principal, scope)-level and persists, per the
  // specification's "revoked (principal, scope) pair" record.
  auto stack = SecStack::delos();
  const PrincipalId principal = stack.authority->register_principal({});
  const Scope scope = Scope::prefix(path({1, 2}));
  const auto first = stack.authority->issue(principal, scope, 0, 100);
  check(first.has_value(), "issuance succeeds");
  stack.authority->revoke(principal, Scope::prefix(path({1, 2})), 20);

  const auto reissued = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 21, 100);
  check(reissued.has_value() && reissued->id != first->id, "re-issuance succeeds");
  if (reissued) {
    check(stack.authority->authorize(*reissued, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 22) ==
              Decision::Deny,
          "a re-issued attestation for a revoked scope stays revoked");
  }
  check(stack.authority->authorize(*first, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 19) ==
            Decision::Allow,
        "revocation is effective only from the recorded epoch");

  // Legacy contrast: revocation removes the entry, and a fresh grant
  // restores authorization.
  auto legacy = SecStack::legacy();
  const PrincipalId l_principal = legacy.authority->register_principal({});
  legacy.authority->issue(l_principal, Scope::prefix(path({1, 2})), 0, 100);
  legacy.authority->revoke(l_principal, Scope::prefix(path({1, 2})), 20);
  const auto l_reissued = legacy.authority->issue(l_principal, Scope::prefix(path({1, 2})), 21, 100);
  check(l_reissued.has_value(), "legacy re-issuance succeeds");
  if (l_reissued) {
    check(legacy.authority->authorize(*l_reissued, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 22) ==
              Decision::Allow,
          "legacy revocation removes entries and permits a fresh grant");
  }
}

}  // namespace

int main() {
  test_delos_seal_binds_epoch_and_principal();
  test_delos_refresh_rejects_non_newer_epoch();
  test_delos_seal_changes_when_any_bound_input_changes();
  test_refresh_epoch_binding_differs_from_legacy();
  test_prefix_scope_covers_longer_paths();
  test_exact_scope_matches_only_identical_path();
  test_delos_revocation_is_effective_from_recorded_epoch();
  test_revocation_key_is_scope_specific();
  test_attestation_lifetime_boundary_is_inclusive();
  test_revoked_scope_stays_revoked_after_reissue();

  if (failures == 0) {
    std::printf("test_delos: 10 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_delos: %d failures\n", failures);
  return 1;
}
