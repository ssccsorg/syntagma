# tagma-sec

Security primitive layer of the synTagma ecosystem. Provides integrity, authorization, audit, and non-repudiation guarantees over Tagma coordinate objects and coordination traffic. Confidentiality is delegated to the hybrid interface described in the specification; the layer claims no confidentiality from coordinate structure.

The definitive contract is `docs/spec/tagma-sec.md`. This crate is the initial implementation (version 0.1.0) that validates the module interfaces and the security model at code level. The legacy pattern in this crate serves exclusively as a reverse-verification mirror. Delos, the composite security engine that wires these primitives with neXus and hybrid confidentiality, is a separate project in planning stage.

## Security model

| Property | Module | Guarantee |
|----------|--------|-----------|
| Integrity | integrity | Tamper-evident binding of records to paths, principals, and epochs |
| Authorization | authority | Decisions over which principal may act on which scope |
| Audit | audit | Append-only, chained, externally verifiable evidence log |
| Non-repudiation | channel | Signed evidence exchange; origin and receipt provable to third parties |
| Confidentiality | hybrid (external) | Provided by hybrid encryption composed with the envelope, not by structure |

The layer follows the repositioning in the specification. The initial framing proposed coordinate structure as a substitute for encryption. Four weaknesses led to its rejection: published interpretation rules collapse confidentiality, a captured CoordPath can be replayed, the valid subspace is enumerable by construction, and composition lacks any keyed transformation. **Security rests on keyed primitives over public arithmetic; Tagma contributes collision-free addressing, not secrecy.** The adversary may observe and replay any path and knows the public arithmetic, but does not hold the keys issued by the authority. The audit log is append-only; chaining makes silent retroactive modification detectable.

## Crate layout

| Module | Role |
|--------|------|
| `types.rs` | Shared types: PrincipalId, Epoch, Action, Decision, Scope (Exact/Prefix), Path, Attestation, Seal, Event, SignedEvidence, Receipt |
| `authority.rs` | Authority trait: register, issue, authorize, revoke |
| `integrity.rs` | Integrity trait: seal, verify, refresh |
| `audit.rs` | Audit trait: append, verify_chain, prove, export; ChainedAudit, InclusionProof, EvidenceBundle |
| `channel.rs` | Channel trait: sign, verify, exchange, verify_receipt; MacChannel |
| `legacy.rs` | Legacy pattern mirror: ACL authority, record-and-path seal, immediate revocation |
| `delos.rs` | tagma-sec pattern: CoordPath scope matching, four-way seal binding, epoch-scoped revocation |
| `proxy.rs` | SecStack composition and the route_update workflow |
| `hash.rs` | Keyed tag helper over blake3 |

## Module interfaces

### authority

Registration, issuance, authorization, and revocation. Scope matching follows the milestone 1 contract: Exact and Prefix rules over path sequences, with no dependence on path secrecy. Attestations carry an issued epoch and a validity window; authorization checks the window and the epoch-scoped revocation record.

### integrity

Seal, verify, and refresh. The seal is a keyed commitment that does not hide the record. The tagma-sec pattern binds record, path, principal, and epoch, so replay across epochs is detected. Refresh re-binds an existing seal to a newer epoch and returns `None` when the input seal does not verify.

### audit

Append, verify_chain, prove, and export. Every entry chains to its predecessor. Inclusion proofs and evidence bundles pair events with their payloads so an external party can verify commitments without access to the log.

### channel

Sign, verify, exchange, and verify_receipt. Exchanged evidence binds the local payload, the remote principal, and the epoch, so replay of signed messages is prevented. Receipts prove delivery and origin.

## Composition and workflow

`SecStack` composes the four modules behind trait objects and switches between `legacy()` and `delos()`. The `route_update` workflow exercises all four modules: authorization of the path and action at the current epoch, seal and verify of the record, audit commit of the record, channel receipt exchange binding the record to its path, epoch, and origin, and audit commit of the signed evidence. It returns `None` on any rejected step.

## Reverse verification

The shared suite `tests/workflow.rs` runs identically against both stacks. A security property holds for the tagma-sec pattern only if the same contract passes on the legacy mirror. The distinguishing suite `tests/delos.rs` pins the deltas: epoch and principal bound seals, refresh epoch binding, prefix scope coverage, revocation effective from the recorded epoch, scope-specific revocation keys, and the attestation lifetime boundary.

## Development plan

The specification maps the work into milestones. Milestones 1 to 4 (authority, integrity, audit, channel) are implemented in this crate. Milestone 5 (Delos: neXus envelope, hybrid confidentiality composition, wiring) is a separate future project.

## Usage

```rust
use tagma_core::Coord;
use tagma_sec::proxy::{route_update, SecStack};
use tagma_sec::types::{Path, Scope};

let mut stack = SecStack::delos();
let principal = stack.authority.register(b"coordinator-1");
let scope = Scope::Prefix(vec![Coord::new(1).unwrap(), Coord::new(2).unwrap()]);
let att = stack.authority.issue(principal, scope, 0, 100).unwrap();
let path: Path = vec![
    Coord::new(1).unwrap(),
    Coord::new(2).unwrap(),
    Coord::new(3).unwrap(),
];

let res = route_update(&mut stack, &att, &path, b"route-v1", 50).expect("authorized update");
assert!(stack.channel.verify_receipt(&res.receipt));
```

## Testing

From `sw/rust`:

```text
cargo test -p tagma-sec
cargo test --workspace
```

The crate carries 26 tests: 17 common workflow tests and 9 tagma-sec-specific property tests. The README usage example is additionally compiled as a doctest.

## Known limitations

The initial implementation is a proof of concept, not a production deployment. Specific boundaries:

- The channel is MAC-based; it authenticates origin between holders of the shared key. True non-repudiation against the key holder requires an asymmetric signature under a published verification key.
- PoC keys are hardcoded constants and must be externalized.
- Paths are `Vec<Coord>`; the allocator-free `CoordPath<N>` representation with a depth bound of 19 is deferred.
- The concrete epoch source is an open question in the specification; the interfaces do not depend on its realization.
- Receipt verification parses a fixed 16-byte tail as remote and epoch; an explicit envelope format is future work.
- Identity issuance does not bind presented credentials; identifier management is out of scope for the specification.

---

## References

- `docs/spec/tagma-sec.md`: Tagma Security Layer Specification, version 0.2 draft
- `docs/spec/coord-space.md`: coordinate space specification
