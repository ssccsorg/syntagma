//! Common scenario tests for the route-update workflow.
//!
//! The same requirements are exercised against both the legacy and the
//! tagma-sec stacks through the `SecStack` proxy, making the stack swap the
//! contract under test. Any behavior asserted here must hold for both
//! implementations.

use tagma_core::Coord;
use tagma_sec::proxy::{route_update, SecStack};
use tagma_sec::types::{Attestation, Decision, Path, Scope, ACTION_ROUTE_UPDATE};

/// Builds a path from raw coordinate indices.
fn path(idxs: &[u16]) -> Path {
    idxs.iter()
        .map(|&i| Coord::new(i).expect("valid coord"))
        .collect()
}

/// Issues an attestation valid from epoch 0 through epoch 100.
fn issue_default(stack: &mut SecStack, principal: u64, scope: Scope) -> Attestation {
    stack
        .authority
        .issue(principal, scope, 0, 100)
        .expect("issuance succeeds")
}

/// Runs the given closure against both stack implementations.
fn with_both_stacks<F: Fn(&mut SecStack)>(f: F) {
    let mut legacy = SecStack::legacy();
    f(&mut legacy);
    let mut delos = SecStack::delos();
    f(&mut delos);
}

#[test]
fn authorized_route_update_is_accepted_and_chained() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = issue_default(stack, principal, Scope::Prefix(path(&[1, 2])));

        let first = route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50)
            .expect("in-scope update accepted");
        assert_eq!(first.event.id, 0);
        assert_eq!(first.event.prev, None);

        let second = route_update(stack, &att, &path(&[1, 2, 4]), b"route-v2", 51)
            .expect("second in-scope update accepted");
        assert_eq!(
            second.event.id, 2,
            "the receipt entry of the first update takes id 1"
        );
        assert_eq!(second.event.prev, Some(1));

        assert!(
            stack.audit.verify_chain(0, second.event.id),
            "chain is intact"
        );
        assert!(
            stack.audit.verify_chain(0, 0),
            "single-entry chain is intact"
        );
    });
}

#[test]
fn route_outside_scope_is_rejected_without_appending() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = issue_default(stack, principal, Scope::Exact(path(&[1, 2])));

        // A longer path is outside an Exact scope.
        assert!(route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50).is_none());

        // The exact path is accepted afterwards, and the chain starts clean.
        let accepted = route_update(stack, &att, &path(&[1, 2]), b"route-v1", 50)
            .expect("exact path accepted");
        assert_eq!(accepted.event.id, 0);
        assert!(stack.audit.verify_chain(0, 1));
    });
}

#[test]
fn expired_attestation_is_rejected() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 0, 10)
            .expect("issuance succeeds");

        assert!(route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 10).is_some());
        assert!(
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 11).is_none(),
            "expired attestation must be rejected"
        );
    });
}

#[test]
fn attestation_not_yet_valid_is_rejected() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 10, 100)
            .expect("issuance succeeds");

        assert!(
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 9).is_none(),
            "attestation must not authorize before its issued epoch"
        );
        assert!(route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 10).is_some());
    });
}

#[test]
fn malformed_attestation_lifetime_never_authorizes() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        // An invalid window: the issued epoch lies after the validity end.
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 100, 10)
            .expect("issuance succeeds");

        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 50),
            Decision::Deny,
            "an invalid lifetime window must fail closed at every epoch"
        );
        assert!(route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50).is_none());
    });
}

#[test]
fn principals_are_scoped_independently() {
    with_both_stacks(|stack| {
        let alice = stack.authority.register(b"coordinator-alice");
        let bob = stack.authority.register(b"coordinator-bob");
        let alice_att = issue_default(stack, alice, Scope::Prefix(path(&[1, 2])));
        let bob_att = issue_default(stack, bob, Scope::Prefix(path(&[5, 6])));

        assert!(
            route_update(stack, &alice_att, &path(&[5, 6]), b"route-bob", 50).is_none(),
            "alice must not act on bob's scope"
        );
        assert!(
            route_update(stack, &bob_att, &path(&[1, 2]), b"route-alice", 50).is_none(),
            "bob must not act on alice's scope"
        );

        let res = route_update(stack, &alice_att, &path(&[1, 2, 3]), b"route-alice", 50)
            .expect("alice update accepted");
        assert_eq!(res.receipt.remote, alice);
    });
}

#[test]
fn revoked_scope_stops_authorization() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let scope = Scope::Prefix(path(&[1, 2]));
        let att = stack
            .authority
            .issue(principal, scope.clone(), 0, 100)
            .expect("issuance succeeds");

        assert_eq!(
            stack.authority.authorize(&att, &path(&[1, 2, 3]), 1, 5),
            Decision::Allow
        );

        stack.authority.revoke(principal, &scope, 10);
        assert!(
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 10).is_none(),
            "revoked scope must stop authorization at the recorded epoch"
        );
    });
}

#[test]
fn tampered_record_fails_integrity_verification() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let record = b"route-v1";
        let target = path(&[1, 2, 3]);

        let seal = stack.integrity.seal(record, &target, principal, 50);
        assert!(stack
            .integrity
            .verify(record, &target, principal, 50, &seal));

        assert!(
            !stack
                .integrity
                .verify(b"route-v1-tampered", &target, principal, 50, &seal),
            "altered record must fail verification"
        );
    });
}

#[test]
fn integrity_refresh_rebinds_seal() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let record = b"route-v1";
        let target = path(&[1, 2]);

        let seal = stack.integrity.seal(record, &target, principal, 5);
        let refreshed = stack
            .integrity
            .refresh(record, &target, principal, 5, 6, &seal)
            .expect("refresh from the binding epoch succeeds");
        assert!(
            stack
                .integrity
                .verify(record, &target, principal, 6, &refreshed),
            "refreshed seal verifies at the new epoch"
        );
    });
}

#[test]
fn channel_detects_tampered_evidence() {
    with_both_stacks(|stack| {
        let evidence = b"route-update-request";
        let signed = stack.channel.sign(evidence);
        assert!(stack.channel.verify(&signed));

        let mut forged = signed.clone();
        forged.evidence[0] ^= 0xFF;
        assert!(
            !stack.channel.verify(&forged),
            "altered evidence must fail channel verification"
        );
    });
}

#[test]
fn channel_exchange_binds_epoch_and_remote() {
    with_both_stacks(|stack| {
        let receipt = stack.channel.exchange(b"request", 7, 5);
        assert!(stack.channel.verify_receipt(&receipt));
        assert_eq!(receipt.remote, 7);
        assert_eq!(receipt.epoch, 5);

        let mut forged_epoch = receipt.clone();
        forged_epoch.epoch = 6;
        assert!(
            !stack.channel.verify_receipt(&forged_epoch),
            "epoch tampering must be detected"
        );

        let mut forged_remote = receipt.clone();
        forged_remote.remote = 8;
        assert!(
            !stack.channel.verify_receipt(&forged_remote),
            "remote tampering must be detected"
        );
    });
}

#[test]
fn workflow_produces_verifiable_receipt() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = issue_default(stack, principal, Scope::Prefix(path(&[1, 2])));

        let res =
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50).expect("update accepted");
        assert!(
            stack.channel.verify_receipt(&res.receipt),
            "workflow receipt verifies through the channel"
        );
        assert_eq!(res.receipt.remote, principal);
        assert_eq!(res.receipt.epoch, 50);

        let mut forged = res.receipt.clone();
        forged.epoch = 51;
        assert!(
            !stack.channel.verify_receipt(&forged),
            "tampered receipt must fail verification"
        );
    });
}

#[test]
fn audit_chain_rejects_invalid_ranges() {
    with_both_stacks(|stack| {
        stack.audit.append(b"event-0", 0);
        stack.audit.append(b"event-1", 1);
        stack.audit.append(b"event-2", 2);

        assert!(stack.audit.verify_chain(0, 2));
        assert!(
            !stack.audit.verify_chain(2, 1),
            "reversed range must be rejected"
        );
        assert!(
            !stack.audit.verify_chain(0, 3),
            "range past the end must be rejected"
        );
    });
}

#[test]
fn audit_proof_and_export_verify() {
    with_both_stacks(|stack| {
        stack.audit.append(b"event-0", 0);
        stack.audit.append(b"event-1", 1);
        stack.audit.append(b"event-2", 2);

        let proof = stack.audit.prove(1).expect("inclusion proof");
        assert!(proof.verify(), "proof verifies without the log");

        let bundle = stack.audit.export(0, 2).expect("evidence bundle");
        assert!(bundle.verify(), "bundle verifies without the log");

        let mid = stack.audit.export(1, 2).expect("mid-range evidence bundle");
        assert!(
            mid.verify(),
            "mid-range bundle verifies internally despite an external prev link"
        );

        let mut forged = bundle.clone();
        forged.entries[1].payload = b"tampered".to_vec();
        assert!(
            !forged.verify(),
            "tampered payload breaks bundle verification"
        );

        assert!(
            stack.audit.prove(3).is_none(),
            "out-of-range proof rejected"
        );
        assert!(
            stack.audit.export(2, 1).is_none(),
            "reversed range rejected"
        );
    });
}

#[test]
fn audit_event_commits_to_record_payload() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = issue_default(stack, principal, Scope::Prefix(path(&[1, 2])));

        let record = b"route-v1";
        let res =
            route_update(stack, &att, &path(&[1, 2, 3]), record, 50).expect("update accepted");
        assert_eq!(
            &res.event.payload_hash,
            blake3::hash(record).as_bytes(),
            "audit entry commits to the record payload"
        );
    });
}
