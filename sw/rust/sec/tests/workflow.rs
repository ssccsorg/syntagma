//! Common scenario tests for the route-update workflow.
//!
//! The same requirements are exercised against both the legacy and the
//! tagma-sec stacks through the `SecStack` proxy, making the stack swap the
//! contract under test. Any behavior asserted here must hold for both
//! implementations.

use tagma_core::Coord;
use tagma_sec::proxy::{route_update, SecStack};
use tagma_sec::types::{Decision, Path, Scope};

/// Builds a path from raw coordinate indices.
fn path(idxs: &[u16]) -> Path {
    idxs.iter()
        .map(|&i| Coord::new(i).expect("valid coord"))
        .collect()
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
        let scope = Scope::Prefix(path(&[1, 2]));
        let att = stack
            .authority
            .issue(principal, scope, 100)
            .expect("issuance succeeds");

        let first = route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50)
            .expect("in-scope update accepted");
        assert_eq!(first.event.id, 0);
        assert_eq!(first.event.prev, None);

        let second = route_update(stack, &att, &path(&[1, 2, 4]), b"route-v2", 51)
            .expect("second in-scope update accepted");
        assert_eq!(
            second.event.id, 2,
            "receipt entry of the first update takes id 1"
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
        let scope = Scope::Exact(path(&[1, 2]));
        let att = stack
            .authority
            .issue(principal, scope, 100)
            .expect("issuance succeeds");

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
            .issue(principal, Scope::Prefix(path(&[1, 2])), 10)
            .expect("issuance succeeds");

        assert!(route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 10).is_some());
        assert!(
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 11).is_none(),
            "expired attestation must be rejected"
        );
    });
}

#[test]
fn revoked_scope_stops_authorization() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let scope = Scope::Prefix(path(&[1, 2]));
        let att = stack
            .authority
            .issue(principal, scope.clone(), 100)
            .expect("issuance succeeds");

        assert_eq!(
            stack.authority.authorize(&att, &path(&[1, 2, 3]), 5),
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
fn workflow_produces_verifiable_signed_evidence() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 100)
            .expect("issuance succeeds");

        let res =
            route_update(stack, &att, &path(&[1, 2, 3]), b"route-v1", 50).expect("update accepted");
        assert!(
            stack.channel.verify(&res.signed),
            "non-repudiation evidence verifies through the channel"
        );

        let mut forged = res.signed.clone();
        forged.evidence[0] ^= 0xFF;
        assert!(
            !stack.channel.verify(&forged),
            "tampered evidence must fail channel verification"
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
fn audit_event_commits_to_record_payload() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 100)
            .expect("issuance succeeds");

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
