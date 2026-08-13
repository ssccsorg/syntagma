//! Real-scenario tests for the route-update workflow.
//!
//! These pin the expected behavior of complete journeys and investigation
//! flows. The expectation is the contract: a failure means the
//! implementation must change, the test stays. Every scenario runs on both
//! stacks through the mirror pattern.

use tagma_core::Coord;
use tagma_sec::proxy::{route_update, SecStack};
use tagma_sec::types::{Decision, Path, Scope, ACTION_ROUTE_UPDATE};

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
fn route_update_journey_is_identical_on_both_stacks() {
    with_both_stacks(|stack| {
        let alice = stack.authority.register(b"coordinator-alice");
        let bob = stack.authority.register(b"coordinator-bob");
        let alice_scope = Scope::Prefix(path(&[1, 2]));
        let alice_att = stack
            .authority
            .issue(alice, alice_scope.clone(), 0, 60)
            .expect("issuance succeeds");
        let bob_att = stack
            .authority
            .issue(bob, Scope::Exact(path(&[9, 9])), 0, 100)
            .expect("issuance succeeds");

        let r1 = route_update(stack, &alice_att, &path(&[1, 2, 3]), b"r1", 10)
            .expect("alice update in scope");
        assert_eq!(r1.event.id, 0);
        assert_eq!(r1.receipt.remote, alice);

        // Cross-principal denial: bob cannot act in alice's subtree.
        assert!(route_update(stack, &bob_att, &path(&[1, 2, 4]), b"r2", 11).is_none());

        // Mid-journey revocation: alice loses the scope at epoch 20.
        stack.authority.revoke(alice, &alice_scope, 20);
        assert!(route_update(stack, &alice_att, &path(&[1, 2, 5]), b"r3", 21).is_none());

        // Expiry: bob's attestation dies at epoch 100.
        assert!(route_update(stack, &bob_att, &path(&[9, 9]), b"r4", 101).is_none());

        // Denials append nothing: the log holds exactly the two entries of
        // the one accepted update.
        assert!(stack.audit.verify_chain(0, 1));
        assert!(stack.audit.prove(2).is_none(), "denials append no events");
        assert!(stack.audit.export(0, 1).expect("evidence bundle").verify());
    });
}

#[test]
fn audit_investigation_flow_proves_who_did_what() {
    with_both_stacks(|stack| {
        let alice = stack.authority.register(b"coordinator-alice");
        let bob = stack.authority.register(b"coordinator-bob");
        let alice_att = stack
            .authority
            .issue(alice, Scope::Prefix(path(&[1, 2])), 0, 100)
            .expect("issuance succeeds");
        let bob_att = stack
            .authority
            .issue(bob, Scope::Exact(path(&[7, 8])), 0, 100)
            .expect("issuance succeeds");

        let a1 = route_update(stack, &alice_att, &path(&[1, 2, 3]), b"route-a", 10).unwrap();
        let b1 = route_update(stack, &bob_att, &path(&[7, 8]), b"route-b", 12).unwrap();

        // Offline investigation: the exported bundle verifies without the
        // log, and each receipt binds its origin and its epoch.
        let bundle = stack.audit.export(0, 3).expect("full evidence bundle");
        assert!(bundle.verify());

        assert!(stack.channel.verify_receipt(&a1.receipt));
        assert_eq!(a1.receipt.remote, alice);
        assert_eq!(a1.receipt.epoch, 10);
        assert!(stack.channel.verify_receipt(&b1.receipt));
        assert_eq!(b1.receipt.remote, bob);
        assert_eq!(b1.receipt.epoch, 12);

        // Replay resistance: a later update produces different signed
        // evidence, so a captured artifact is stale by construction.
        let a2 = route_update(stack, &alice_att, &path(&[1, 2, 4]), b"route-a2", 20).unwrap();
        assert_ne!(a2.receipt.signed.evidence, a1.receipt.signed.evidence);
        assert_ne!(a2.receipt.signed.tag, a1.receipt.signed.tag);
        assert_eq!(a2.receipt.epoch, 20);
    });
}

#[test]
fn single_epoch_window_is_exact() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[1, 2])), 10, 10)
            .expect("issuance succeeds");

        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 9),
            Decision::Deny
        );
        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 10),
            Decision::Allow,
            "a single-epoch window is valid at exactly its one epoch"
        );
        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 11),
            Decision::Deny
        );
    });
}

#[test]
fn empty_prefix_scope_authorizes_every_path() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[])), 0, 100)
            .expect("issuance succeeds");

        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 50),
            Decision::Allow,
            "an empty prefix scope is the root scope"
        );
        assert_eq!(
            stack
                .authority
                .authorize(&att, &path(&[7, 8]), ACTION_ROUTE_UPDATE, 50),
            Decision::Allow
        );
    });
}

#[test]
fn deep_path_scenario_reaches_the_depth_bound() {
    with_both_stacks(|stack| {
        let principal = stack.authority.register(b"coordinator-1");
        let att = stack
            .authority
            .issue(principal, Scope::Prefix(path(&[0])), 0, 100)
            .expect("issuance succeeds");

        // 19 coords matches the maximum scope depth in the specification.
        let deep: Path = (0..19)
            .map(|i| Coord::new(i).expect("valid coord"))
            .collect();
        let res = route_update(stack, &att, &deep, b"deep-route", 50).expect("deep path update");
        assert_eq!(res.receipt.epoch, 50);
        assert!(stack.audit.verify_chain(0, 1));
    });
}
