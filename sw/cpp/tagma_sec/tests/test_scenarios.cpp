// Real-scenario tests for the route-update workflow. These pin the expected
// behavior of complete journeys and investigation flows. The expectation is
// the contract: a failure means the implementation must change, the test
// stays. Every scenario runs on both stacks through the mirror pattern.
// Mirrors sw/rust/sec/tests/scenarios.rs.

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

template <typename F>
void with_both_stacks(F&& f) {
  auto legacy = SecStack::legacy();
  f(legacy);
  auto delos = SecStack::delos();
  f(delos);
}

void test_route_update_journey_is_identical_on_both_stacks() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId alice = stack.authority->register_principal({});
    const PrincipalId bob = stack.authority->register_principal({});
    const Scope alice_scope = Scope::prefix(path({1, 2}));
    const auto alice_att = stack.authority->issue(alice, alice_scope, 0, 60);
    const auto bob_att = stack.authority->issue(bob, Scope::exact(path({9, 9})), 0, 100);
    check(alice_att.has_value() && bob_att.has_value(), "issuance succeeds");

    const auto r1 = route_update(stack, *alice_att, path({1, 2, 3}), bytes("r1"), 10);
    check(r1.has_value() && r1->event.id == 0 && r1->receipt.remote == alice,
          "alice update in scope");

    // Cross-principal denial: bob cannot act in alice's subtree.
    check(!route_update(stack, *bob_att, path({1, 2, 4}), bytes("r2"), 11).has_value(),
          "bob denied in alice's subtree");

    // Mid-journey revocation: alice loses the scope at epoch 20.
    stack.authority->revoke(alice, Scope::prefix(path({1, 2})), 20);
    check(!route_update(stack, *alice_att, path({1, 2, 5}), bytes("r3"), 21).has_value(),
          "alice denied after revocation");

    // Expiry: bob's attestation dies at epoch 100.
    check(!route_update(stack, *bob_att, path({9, 9}), bytes("r4"), 101).has_value(),
          "bob denied after expiry");

    // Denials append nothing: the log holds exactly the two entries of the
    // one accepted update.
    check(stack.audit->verify_chain(0, 1), "chain covers the accepted update");
    check(!stack.audit->prove(2).has_value(), "denials append no events");
    const auto bundle = stack.audit->export_range(0, 1);
    check(bundle.has_value() && bundle->verify(), "evidence bundle verifies");
  });
}

void test_audit_investigation_flow_proves_who_did_what() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId alice = stack.authority->register_principal({});
    const PrincipalId bob = stack.authority->register_principal({});
    const auto alice_att = stack.authority->issue(alice, Scope::prefix(path({1, 2})), 0, 100);
    const auto bob_att = stack.authority->issue(bob, Scope::exact(path({7, 8})), 0, 100);
    check(alice_att.has_value() && bob_att.has_value(), "issuance succeeds");

    const auto a1 = route_update(stack, *alice_att, path({1, 2, 3}), bytes("route-a"), 10);
    const auto b1 = route_update(stack, *bob_att, path({7, 8}), bytes("route-b"), 12);
    check(a1.has_value() && b1.has_value(), "both updates accepted");

    // Offline investigation: the exported bundle verifies without the log,
    // and each receipt binds its origin and its epoch.
    const auto bundle = stack.audit->export_range(0, 3);
    check(bundle.has_value() && bundle->verify(), "full evidence bundle verifies");

    check(stack.channel->verify_receipt(a1->receipt), "alice receipt verifies");
    check(a1->receipt.remote == alice && a1->receipt.epoch == 10, "alice receipt fields");
    check(stack.channel->verify_receipt(b1->receipt), "bob receipt verifies");
    check(b1->receipt.remote == bob && b1->receipt.epoch == 12, "bob receipt fields");

    // Replay resistance: a later update produces different signed evidence,
    // so a captured artifact is stale by construction.
    const auto a2 = route_update(stack, *alice_att, path({1, 2, 4}), bytes("route-a2"), 20);
    check(a2.has_value(), "second alice update accepted");
    if (a2) {
      check(a2->receipt.signed_evidence.evidence != a1->receipt.signed_evidence.evidence,
            "evidence differs across updates");
      check(a2->receipt.signed_evidence.tag != a1->receipt.signed_evidence.tag,
            "tags differ across updates");
      check(a2->receipt.epoch == 20, "second receipt binds its epoch");
    }
  });
}

void test_single_epoch_window_is_exact() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 10, 10);
    check(att.has_value(), "issuance succeeds");

    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 9) ==
              Decision::Deny,
          "before the single epoch denied");
    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 10) ==
              Decision::Allow,
          "the single epoch is valid");
    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 11) ==
              Decision::Deny,
          "after the single epoch denied");
  });
}

void test_empty_prefix_scope_authorizes_every_path() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::prefix(path({})), 0, 100);
    check(att.has_value(), "issuance succeeds");

    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 50) ==
              Decision::Allow,
          "an empty prefix scope is the root scope");
    check(stack.authority->authorize(*att, path({7, 8}), ACTION_ROUTE_UPDATE, 50) ==
              Decision::Allow,
          "root scope covers any path");
  });
}

void test_deep_path_scenario_reaches_the_depth_bound() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::prefix(path({0})), 0, 100);
    check(att.has_value(), "issuance succeeds");

    // 19 coords matches the maximum scope depth in the specification.
    Path deep;
    for (uint16_t i = 0; i < 19; ++i) deep.push_back(*tagma::Coord::from_index(i));
    const auto res = route_update(stack, *att, deep, bytes("deep-route"), 50);
    check(res.has_value() && res->receipt.epoch == 50, "deep path update accepted");
    check(stack.audit->verify_chain(0, 1), "deep path chain intact");
  });
}

}  // namespace

int main() {
  test_route_update_journey_is_identical_on_both_stacks();
  test_audit_investigation_flow_proves_who_did_what();
  test_single_epoch_window_is_exact();
  test_empty_prefix_scope_authorizes_every_path();
  test_deep_path_scenario_reaches_the_depth_bound();

  if (failures == 0) {
    std::printf("test_scenarios: 5 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_scenarios: %d failures\n", failures);
  return 1;
}
