//! tagma-sec-specific properties of the Delos pattern stack.
//!
//! These tests pin the properties that distinguish the tagma-sec pattern
//! from the legacy one: epoch-bound seals, prefix scope semantics, and
//! revocation that is effective only from the recorded epoch.

use tagma_core::Coord;
use tagma_sec::proxy::SecStack;
use tagma_sec::types::{Decision, Path, Scope, ACTION_ROUTE_UPDATE};

/// Builds a path from raw coordinate indices.
fn path(idxs: &[u16]) -> Path {
    idxs.iter()
        .map(|&i| Coord::new(i).expect("valid coord"))
        .collect()
}

#[test]
fn delos_seal_binds_epoch_and_principal() {
    let record = b"route-v1";
    let target = path(&[1, 2]);

    // Legacy seal binds only record and path: principal and epoch are free.
    let legacy = SecStack::legacy();
    let legacy_seal = legacy.integrity.seal(record, &target, 7, 5);
    assert!(legacy.integrity.verify(record, &target, 7, 5, &legacy_seal));
    assert!(
        legacy
            .integrity
            .verify(record, &target, 99, 99, &legacy_seal),
        "legacy seal ignores principal and epoch"
    );

    // Delos seal binds record, path, principal, and epoch.
    let delos = SecStack::delos();
    let delos_seal = delos.integrity.seal(record, &target, 7, 5);
    assert!(delos.integrity.verify(record, &target, 7, 5, &delos_seal));
    assert!(
        !delos.integrity.verify(record, &target, 7, 6, &delos_seal),
        "epoch change must break the delos seal"
    );
    assert!(
        !delos.integrity.verify(record, &target, 8, 5, &delos_seal),
        "principal change must break the delos seal"
    );
    assert!(
        !delos
            .integrity
            .verify(record, &path(&[1, 3]), 7, 5, &delos_seal),
        "path change must break the delos seal"
    );
}

#[test]
fn delos_seal_changes_when_any_bound_input_changes() {
    let delos = SecStack::delos();
    let record = b"route-v1";
    let target = path(&[1, 2]);
    let base = delos.integrity.seal(record, &target, 7, 5);

    let cases: Vec<(&[u8], Path, u64, u64)> = vec![
        (b"route-v2", path(&[1, 2]), 7, 5), // record change
        (record, path(&[1, 3]), 7, 5),      // path change
        (record, path(&[1, 2]), 8, 5),      // principal change
        (record, path(&[1, 2]), 7, 6),      // epoch change
    ];
    for (r, p, principal, epoch) in cases {
        let seal = delos.integrity.seal(r, &p, principal, epoch);
        assert_ne!(
            seal.tag, base.tag,
            "the delos seal must change when any bound input changes"
        );
    }
}

#[test]
fn refresh_epoch_binding_differs_from_legacy() {
    let record = b"route-v1";
    let target = path(&[1, 2]);

    // Legacy refresh accepts any from_epoch and re-emits the identical tag,
    // because the seal never bound the epoch in the first place.
    let legacy = SecStack::legacy();
    let legacy_seal = legacy.integrity.seal(record, &target, 7, 5);
    let legacy_refreshed = legacy
        .integrity
        .refresh(record, &target, 7, 99, 100, &legacy_seal)
        .expect("legacy refresh accepts any from_epoch");
    assert_eq!(
        legacy_refreshed.tag, legacy_seal.tag,
        "legacy seal is epoch-free, so refresh is a no-op"
    );

    // Delos refresh re-binds the seal to the new epoch.
    let delos = SecStack::delos();
    let delos_seal = delos.integrity.seal(record, &target, 7, 5);
    let delos_refreshed = delos
        .integrity
        .refresh(record, &target, 7, 5, 6, &delos_seal)
        .expect("refresh from the binding epoch succeeds");
    assert_ne!(
        delos_refreshed.tag, delos_seal.tag,
        "delos refresh must re-bind the epoch"
    );
    assert!(delos
        .integrity
        .verify(record, &target, 7, 6, &delos_refreshed));
    assert!(
        !delos
            .integrity
            .verify(record, &target, 7, 5, &delos_refreshed),
        "refreshed seal no longer verifies at the old epoch"
    );

    // Delos refresh from a wrong from_epoch is rejected.
    assert!(
        delos
            .integrity
            .refresh(record, &target, 7, 99, 6, &delos_seal)
            .is_none(),
        "refresh from a wrong epoch must be rejected"
    );
}

#[test]
fn prefix_scope_covers_longer_paths() {
    let mut stack = SecStack::delos();
    let principal = stack.authority.register(b"coordinator-1");
    let att = stack
        .authority
        .issue(principal, Scope::Prefix(path(&[1, 2])), 0, 100)
        .expect("issuance succeeds");

    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2]), ACTION_ROUTE_UPDATE, 50),
        Decision::Allow
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 50),
        Decision::Allow
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3, 4]), ACTION_ROUTE_UPDATE, 50),
        Decision::Allow
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 3]), ACTION_ROUTE_UPDATE, 50),
        Decision::Deny
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[2, 3]), ACTION_ROUTE_UPDATE, 50),
        Decision::Deny
    );
}

#[test]
fn exact_scope_matches_only_identical_path() {
    let mut stack = SecStack::delos();
    let principal = stack.authority.register(b"coordinator-1");
    let att = stack
        .authority
        .issue(principal, Scope::Exact(path(&[1, 2])), 0, 100)
        .expect("issuance succeeds");

    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2]), ACTION_ROUTE_UPDATE, 50),
        Decision::Allow
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 50),
        Decision::Deny
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 3]), ACTION_ROUTE_UPDATE, 50),
        Decision::Deny
    );
}

#[test]
fn delos_revocation_is_effective_from_recorded_epoch() {
    let mut stack = SecStack::delos();
    let principal = stack.authority.register(b"coordinator-1");
    let scope = Scope::Prefix(path(&[1, 2]));
    let att = stack
        .authority
        .issue(principal, scope.clone(), 0, 100)
        .expect("issuance succeeds");

    stack.authority.revoke(principal, &scope, 10);

    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 9),
        Decision::Allow,
        "delos revocation is not effective before the recorded epoch"
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 10),
        Decision::Deny,
        "delos revocation takes effect at the recorded epoch"
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 11),
        Decision::Deny
    );

    // Legacy contrast: the ACL entry is removed immediately, so there is no
    // grace window before the recorded epoch.
    let mut legacy = SecStack::legacy();
    let principal = legacy.authority.register(b"coordinator-1");
    let scope = Scope::Prefix(path(&[1, 2]));
    let att = legacy
        .authority
        .issue(principal, scope.clone(), 0, 100)
        .expect("issuance succeeds");
    legacy.authority.revoke(principal, &scope, 10);
    assert_eq!(
        legacy
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 9),
        Decision::Deny,
        "legacy revocation removes the entry immediately"
    );
}

#[test]
fn revocation_key_is_scope_specific() {
    let mut stack = SecStack::delos();
    let principal = stack.authority.register(b"coordinator-1");
    let exact_att = stack
        .authority
        .issue(principal, Scope::Exact(path(&[1, 2])), 0, 100)
        .expect("issuance succeeds");
    let prefix_att = stack
        .authority
        .issue(principal, Scope::Prefix(path(&[1, 2])), 0, 100)
        .expect("issuance succeeds");

    stack
        .authority
        .revoke(principal, &Scope::Exact(path(&[1, 2])), 10);

    assert_eq!(
        stack
            .authority
            .authorize(&exact_att, &path(&[1, 2]), ACTION_ROUTE_UPDATE, 10),
        Decision::Deny,
        "exact scope is revoked"
    );
    assert_eq!(
        stack
            .authority
            .authorize(&prefix_att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 10),
        Decision::Allow,
        "prefix scope has a distinct revocation key"
    );
}

#[test]
fn attestation_lifetime_boundary_is_inclusive() {
    let mut stack = SecStack::delos();
    let principal = stack.authority.register(b"coordinator-1");
    let att = stack
        .authority
        .issue(principal, Scope::Prefix(path(&[1, 2])), 0, 10)
        .expect("issuance succeeds");

    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 10),
        Decision::Allow,
        "the validity end epoch is inclusive"
    );
    assert_eq!(
        stack
            .authority
            .authorize(&att, &path(&[1, 2, 3]), ACTION_ROUTE_UPDATE, 11),
        Decision::Deny
    );
}
