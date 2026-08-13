// Common scenario tests for the route-update workflow. The same
// requirements are exercised against both the legacy and the tagma-sec
// stacks through the SecStack proxy, making the stack swap the contract
// under test. Mirrors sw/rust/sec/tests/workflow.rs.

#include <tagma_sec/proxy.h>

#include <cstdio>
#include <cstring>
#include <stdexcept>
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

// Issues an attestation valid from epoch 0 through epoch 100.
Attestation issue_default(SecStack& stack, PrincipalId principal, Scope scope) {
  auto att = stack.authority->issue(principal, std::move(scope), 0, 100);
  if (!att.has_value()) throw std::runtime_error("issuance failed");
  return *att;
}

// Runs the given closure against both stack implementations.
template <typename F>
void with_both_stacks(F&& f) {
  auto legacy = SecStack::legacy();
  f(legacy);
  auto delos = SecStack::delos();
  f(delos);
}

void test_authorized_route_update_is_accepted_and_chained() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Attestation att = issue_default(stack, principal, Scope::prefix(path({1, 2})));

    const auto first = route_update(stack, att, path({1, 2, 3}), bytes("route-v1"), 50);
    check(first.has_value(), "in-scope update accepted");
    if (first) {
      check(first->event.id == 0, "first event id");
      check(!first->event.prev.has_value(), "first prev none");
    }

    const auto second = route_update(stack, att, path({1, 2, 4}), bytes("route-v2"), 51);
    check(second.has_value(), "second in-scope update accepted");
    if (second) {
      check(second->event.id == 2, "receipt entry of the first update takes id 1");
      check(second->event.prev.has_value() && second->event.prev.value() == 1,
            "second prev 1");
    }
    check(stack.audit->verify_chain(0, second->event.id), "chain is intact");
    check(stack.audit->verify_chain(0, 0), "single-entry chain is intact");
  });
}

void test_route_outside_scope_is_rejected_without_appending() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Attestation att = issue_default(stack, principal, Scope::exact(path({1, 2})));

    // A longer path is outside an Exact scope.
    check(!route_update(stack, att, path({1, 2, 3}), bytes("route-v1"), 50).has_value(),
          "longer path rejected");

    // The exact path is accepted afterwards, and the chain starts clean.
    const auto accepted = route_update(stack, att, path({1, 2}), bytes("route-v1"), 50);
    check(accepted.has_value() && accepted->event.id == 0, "exact path accepted");
    check(stack.audit->verify_chain(0, 1), "record plus receipt chained");
  });
}

void test_expired_attestation_is_rejected() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 0, 10);
    check(att.has_value(), "issuance succeeds");
    check(route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 10).has_value(),
          "valid at the end epoch");
    check(!route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 11).has_value(),
          "expired attestation rejected");
  });
}

void test_attestation_not_yet_valid_is_rejected() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 10, 100);
    check(att.has_value(), "issuance succeeds");
    check(!route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 9).has_value(),
          "before issued epoch rejected");
    check(route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 10).has_value(),
          "valid at the issued epoch");
  });
}

void test_malformed_attestation_lifetime_never_authorizes() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    // An invalid window: the issued epoch lies after the validity end.
    const auto att = stack.authority->issue(principal, Scope::prefix(path({1, 2})), 100, 10);
    check(att.has_value(), "issuance succeeds");
    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 50) ==
              Decision::Deny,
          "invalid lifetime window fails closed");
    check(!route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 50).has_value(),
          "invalid window never authorizes");
  });
}

void test_principals_are_scoped_independently() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId alice = stack.authority->register_principal({});
    const PrincipalId bob = stack.authority->register_principal({});
    const Attestation alice_att =
        issue_default(stack, alice, Scope::prefix(path({1, 2})));
    const Attestation bob_att =
        issue_default(stack, bob, Scope::prefix(path({5, 6})));

    check(!route_update(stack, alice_att, path({5, 6}), bytes("route-bob"), 50).has_value(),
          "alice must not act on bob's scope");
    check(!route_update(stack, bob_att, path({1, 2}), bytes("route-alice"), 50).has_value(),
          "bob must not act on alice's scope");

    const auto res = route_update(stack, alice_att, path({1, 2, 3}), bytes("route-alice"), 50);
    check(res.has_value() && res->receipt.remote == alice, "alice update accepted");
  });
}

void test_authorize_enforces_stored_attestation_window() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const auto att = stack.authority->issue(principal, Scope::exact(path({1, 2})), 0, 10);
    check(att.has_value(), "issuance succeeds");
    // A presented copy with a widened window must not extend the grant.
    Attestation forged = *att;
    forged.valid_until = 1000;
    check(stack.authority->authorize(forged, path({1, 2}), ACTION_ROUTE_UPDATE, 11) ==
              Decision::Deny,
          "authorization uses the stored validity window");
  });
}

void test_authorize_enforces_stored_scope_for_revocation() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Scope scope = Scope::exact(path({1, 2}));
    const auto att = stack.authority->issue(principal, scope, 0, 100);
    check(att.has_value(), "issuance succeeds");
    stack.authority->revoke(principal, Scope::exact(path({1, 2})), 10);
    // A presented copy with a different but matching scope must not bypass
    // the revocation of the granted scope.
    Attestation forged = *att;
    forged.scope = Scope::prefix(path({1, 2}));
    check(stack.authority->authorize(forged, path({1, 2}), ACTION_ROUTE_UPDATE, 10) ==
              Decision::Deny,
          "authorization consults the stored scope's revocation record");
  });
}

void test_revoked_scope_stops_authorization() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Scope scope = Scope::prefix(path({1, 2}));
    const auto att = stack.authority->issue(principal, scope, 0, 100);
    check(att.has_value(), "issuance succeeds");
    check(stack.authority->authorize(*att, path({1, 2, 3}), ACTION_ROUTE_UPDATE, 5) ==
              Decision::Allow,
          "allow before revocation");
    stack.authority->revoke(principal, Scope::prefix(path({1, 2})), 10);
    check(!route_update(stack, *att, path({1, 2, 3}), bytes("route-v1"), 10).has_value(),
          "revoked scope stops authorization at the recorded epoch");
  });
}

void test_tampered_record_fails_integrity_verification() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Bytes record = bytes("route-v1");
    const Path target = path({1, 2, 3});

    const Seal seal = stack.integrity->seal(record, target, principal, 50);
    check(stack.integrity->verify(record, target, principal, 50, seal),
          "seal verifies");
    check(!stack.integrity->verify(bytes("route-v1-tampered"), target, principal, 50, seal),
          "altered record fails verification");
  });
}

void test_integrity_refresh_rebinds_seal() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Bytes record = bytes("route-v1");
    const Path target = path({1, 2});

    const Seal seal = stack.integrity->seal(record, target, principal, 5);
    const auto refreshed = stack.integrity->refresh(record, target, principal, 5, 6, seal);
    check(refreshed.has_value(), "refresh from the binding epoch succeeds");
    if (refreshed) {
      check(stack.integrity->verify(record, target, principal, 6, *refreshed),
            "refreshed seal verifies at the new epoch");
    }
  });
}

void test_channel_detects_tampered_evidence() {
  with_both_stacks([](SecStack& stack) {
    const Bytes evidence = bytes("route-update-request");
    const SignedEvidence signed_evidence = stack.channel->sign(evidence);
    check(stack.channel->verify(signed_evidence), "signed evidence verifies");

    SignedEvidence forged = signed_evidence;
    forged.evidence[0] ^= 0xFF;
    check(!stack.channel->verify(forged), "altered evidence fails verification");
  });
}

void test_channel_exchange_binds_epoch_and_remote() {
  with_both_stacks([](SecStack& stack) {
    const Receipt receipt = stack.channel->exchange(bytes("request"), 7, 5);
    check(stack.channel->verify_receipt(receipt), "receipt verifies");
    check(receipt.remote == 7 && receipt.epoch == 5, "receipt fields");

    Receipt forged_epoch = receipt;
    forged_epoch.epoch = 6;
    check(!stack.channel->verify_receipt(forged_epoch), "epoch tampering detected");

    Receipt forged_remote = receipt;
    forged_remote.remote = 8;
    check(!stack.channel->verify_receipt(forged_remote), "remote tampering detected");
  });
}

void test_workflow_produces_verifiable_receipt() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Attestation att = issue_default(stack, principal, Scope::prefix(path({1, 2})));

    const auto res = route_update(stack, att, path({1, 2, 3}), bytes("route-v1"), 50);
    check(res.has_value(), "update accepted");
    if (res) {
      check(stack.channel->verify_receipt(res->receipt), "workflow receipt verifies");
      check(res->receipt.remote == principal && res->receipt.epoch == 50,
            "receipt binds origin and epoch");
      Receipt forged = res->receipt;
      forged.epoch = 51;
      check(!stack.channel->verify_receipt(forged), "tampered receipt fails");
    }
  });
}

void test_audit_chain_rejects_invalid_ranges() {
  with_both_stacks([](SecStack& stack) {
    stack.audit->append(bytes("event-0"), 0);
    stack.audit->append(bytes("event-1"), 1);
    stack.audit->append(bytes("event-2"), 2);

    check(stack.audit->verify_chain(0, 2), "chain intact");
    check(!stack.audit->verify_chain(2, 1), "reversed range rejected");
    check(!stack.audit->verify_chain(0, 3), "range past the end rejected");
  });
}

void test_audit_proof_and_export_verify() {
  with_both_stacks([](SecStack& stack) {
    stack.audit->append(bytes("event-0"), 0);
    stack.audit->append(bytes("event-1"), 1);
    stack.audit->append(bytes("event-2"), 2);

    const auto proof = stack.audit->prove(1);
    check(proof.has_value() && proof->verify(), "inclusion proof verifies");

    const auto bundle = stack.audit->export_range(0, 2);
    check(bundle.has_value() && bundle->verify(), "evidence bundle verifies");

    const auto mid = stack.audit->export_range(1, 2);
    check(mid.has_value() && mid->verify(),
          "mid-range bundle verifies despite an external prev link");

    if (bundle) {
      EvidenceBundle forged = *bundle;
      forged.entries[1].payload = bytes("tampered");
      check(!forged.verify(), "tampered payload breaks bundle verification");
    }

    check(!stack.audit->prove(3).has_value(), "out-of-range proof rejected");
    check(!stack.audit->export_range(2, 1).has_value(), "reversed range rejected");
  });
}

void test_audit_event_commits_to_record_payload() {
  with_both_stacks([](SecStack& stack) {
    const PrincipalId principal = stack.authority->register_principal({});
    const Attestation att = issue_default(stack, principal, Scope::prefix(path({1, 2})));
    const Bytes record = bytes("route-v1");
    const auto res = route_update(stack, att, path({1, 2, 3}), record, 50);
    check(res.has_value(), "update accepted");
    if (res) {
      check(res->event.payload_hash == sha256(record),
            "audit entry commits to the record payload");
    }
  });
}

}  // namespace

int main() {
  test_authorized_route_update_is_accepted_and_chained();
  test_route_outside_scope_is_rejected_without_appending();
  test_expired_attestation_is_rejected();
  test_attestation_not_yet_valid_is_rejected();
  test_malformed_attestation_lifetime_never_authorizes();
  test_principals_are_scoped_independently();
  test_authorize_enforces_stored_attestation_window();
  test_authorize_enforces_stored_scope_for_revocation();
  test_revoked_scope_stops_authorization();
  test_tampered_record_fails_integrity_verification();
  test_integrity_refresh_rebinds_seal();
  test_channel_detects_tampered_evidence();
  test_channel_exchange_binds_epoch_and_remote();
  test_workflow_produces_verifiable_receipt();
  test_audit_chain_rejects_invalid_ranges();
  test_audit_proof_and_export_verify();
  test_audit_event_commits_to_record_payload();

  if (failures == 0) {
    std::printf("test_workflow: 17 checks passed\n");
    return 0;
  }
  std::fprintf(stderr, "test_workflow: %d failures\n", failures);
  return 1;
}
