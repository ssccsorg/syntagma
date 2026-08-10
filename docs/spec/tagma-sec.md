# Tagma Security Layer Specification (tagma-sec)

Status: draft 0.2
Scope: synTagma security layer, tagma-sec primitives and the Delos composite engine.

This document fixes the security claims of the layer, defines the four module interfaces (authority, integrity, audit, channel), assigns confidentiality to a hybrid interface, and states the neXus integration contract. Interface signatures are provisional until the first `sec` crate lands.

## Purpose

`tagma-sec` is the security primitive layer of the synTagma ecosystem. `Delos` is the composite security engine built from these primitives and composed with external cryptographic material. The layer provides integrity, authorization, audit, and non-repudiation guarantees over Tagma coordinate objects and coordination traffic. It does not claim confidentiality from the structure of the coordinate space; confidentiality is delegated to the hybrid interface defined in this document.

## Terminology

| Term | Meaning |
|------|---------|
| Principal | An entity (device, process, node) with an identity issued by an authority |
| Attestation | A capability statement binding a principal to a scope, carrying an issued epoch and a validity window |
| Scope | A `CoordPath` or a set of `CoordPath` values that an action targets, matched by the rules in the authority section |
| Seal | An integrity binding over a record, its path, and an epoch |
| Evidence | An auditable, verifiable record of an event or an exchange |
| Epoch | A monotonic counter that scopes replay protection |
| Coordination layer | The synTagma topology, routing, and distributed resolver layer (neXus) |

## Repositioning

The initial Delos framing proposed Tagma coordinates as a substitute for encryption. Analysis identified four engineering weaknesses in that framing. Each is recorded here as a rejected claim, with the reasoning that led to its rejection.

1. Published interpretation rules collapse confidentiality. If the mapping from application semantics to Coord slots is public, an observer who captures a `CoordPath` recovers its meaning without breaking any key. The composition formula and linearization are public arithmetic; secrecy cannot rest on them.
2. CoordPath replay. A captured `CoordPath` is a static sequence of structurally valid Coords. Nothing in the coordinate itself binds it to a time, a principal, or a session, so a captured path can be replayed.
3. Brute force over the invalid space. The 16-bit space holds 11,172 valid values and 54,364 invalid values. The valid subspace is enumerable by construction, so structural validity alone cannot serve as a secret.
4. The encryption-substitute claim does not guarantee confidentiality. A keyed transformation is absent from composition, decomposition, and linearization, so those operations cannot satisfy a confidentiality requirement on their own.

Analytic conclusion: the defensible properties of the layer are integrity, authorization, audit, and non-repudiation. The structural layer claims no confidentiality property. Confidentiality is provided by the hybrid interface (Section `Hybrid confidentiality`).

Naming note: `tagma-proof` was considered for the audit and proof portion only; it was set aside as too narrow for the full layer, and the layer keeps the `tagma-sec` name.

Feasibility basis: the rejection follows from the Kerckhoffs principle. The composition formula, the linearization rule, and the validity bounds are public specifications, and a hardware decoder must remain verifiable against public rules. Public arithmetic cannot serve as a secret ingredient. The invalid subspace is enumerable: the 16-bit space holds 65,536 values in total, so a single Coord is fully enumerable. The layer therefore builds on keyed primitives, and Tagma contributes collision-free addressing rather than structural secrecy.

## Relation to the initial Delos framing

The initial framing proposed mechanisms that this repositioning removed or replaced. Each mechanism and its fate:

| Mechanism | Fate | Reason |
|-----------|------|--------|
| Interpreter: maps an invalid Coord to a Fact under interpretation authority | Removed | Interpretation is public arithmetic; anyone can perform it, so it cannot carry secrecy |
| Invalid-space placement in the hybrid cipher | Removed | Placing ciphertext at public, enumerable coordinates adds no secrecy and complicates storage |
| Path-based interpretation authority (`can_interpret`) | Replaced | Authorization rests solely on signed attestations over scopes |
| Sub-10 ns total verification claim | Not a requirement | Coordinate arithmetic is nanosecond-scale; keyed hash, MAC, and signature costs dominate the total |
| Quantum immunity claim | Conditional | Resistance depends on the chosen signature algorithm; post-quantum candidates are listed in Open questions |
| `tagma-id` layer | Out of scope | The spec assumes a principal identity source for `authority.register`; identifier management belongs to a separate spec |

## Security model

### Properties

| Property | Module | Guarantee |
|----------|--------|-----------|
| Integrity | integrity | Tamper-evident binding of records to paths, principals, and epochs |
| Authorization | authority | Decisions over which principal may act on which scope |
| Audit | audit | Append-only, chained, externally verifiable evidence log |
| Non-repudiation | channel | Signed evidence exchange; origin and receipt provable to third parties |
| Confidentiality | hybrid (external) | Provided by hybrid encryption composed with the envelope, not by structure |

### Adversary assumptions

- The adversary can observe, capture, and replay any `CoordPath` and any envelope in transit.
- The adversary knows the composition formula, the linearization rule, and the validity bounds. These are public by design.
- The adversary does not hold private keys issued by the authority and cannot forge signatures under the active verification keys.
- The audit log storage layer is append-only; a log node may be compromised, but the chaining makes silent retroactive modification detectable.

### Trust boundaries

- The authority module is the trust root for principal identity and scope authorization.
- Epoch sources are shared between the integrity and channel modules; their concrete realization is an open question (Section `Open questions`).
- The audit log and the channel module assume an authenticated time or epoch source for ordering.

### Performance expectation

Coordinate arithmetic operations (composition, decomposition, slot access) run at nanosecond scale. Keyed hashing, MAC, and signature verification dominate end-to-end cost, so this specification sets no total-verification bound. Benchmark figures cover the arithmetic layer only.

## Path representation

Scopes are expressed with the `tagma-core` `CoordPath<N>` type, where `N` is a compile-time depth. Variable-length scopes are bounded by a maximum depth instead of dynamic allocation, so the security core stays usable without an allocator, consistent with `tagma-core`.

- Exact and prefix matching operate on the coordinate sequence directly; no hashing or indirection is involved.
- A scope that must cover paths of different depths is represented as a set of fixed-depth paths, one per depth.
- The maximum scope depth is 19 coords, matching the coordinate space that covers 2^256 values with 19 coordinates (Section 3 of `coord-space.md`).

## Module interfaces

Each module lists operations in a language-independent form. Rust trait sketches are informative drafts, marked as such.

### authority

Purpose: principal registration, attestation issuance, scope authorization, and revocation.

Scope is expressed as a `CoordPath` predicate. Authorization decisions depend only on the attestation and the scope condition. Path values are observable and replayable, so the decision never depends on path secrecy.

Scope matching for the initial implementation supports two rules:

| Rule | Meaning | Example |
|------|---------|---------|
| Exact | The target path equals the scope path | scope `CoordPath<2>(a, b)` authorizes only the path `(a, b)` |
| Prefix | The scope path is a prefix of the target path | scope `CoordPath<2>(a, b)` authorizes `(a, b, c)` and any longer path beginning with `(a, b)` |

A scope may name a single path or a set of paths. Compound policies (negation, union of disjoint prefixes, regular expressions over paths) are deferred; the two rules above are the Milestone 1 contract.

Attestations carry a lifetime: an issued epoch and a validity window. `authorize` checks the current epoch against the window and rejects expired attestations. `revoke` records a revoked (principal, scope) pair, effective from the epoch recorded by the authority. The concrete epoch source is an open question (Section `Open questions`); the interface does not depend on its realization.

Operations:

| Operation | Inputs | Outputs | Guarantee |
|-----------|--------|---------|-----------|
| register | Principal credentials | PrincipalId | Identity is unique and bound to the presented credentials |
| issue | PrincipalId, Scope | Attestation | Attestation names exactly the granted scope |
| authorize | Attestation, Action, Scope | Decision | Decision is derived only from the attestation and the scope |
| revoke | PrincipalId, Scope | Result | Attestations for the scope cease to authorize |

Informative draft:

```rust
pub trait Authority {
    fn register(&mut self, credentials: &Credentials) -> Result<PrincipalId, SecError>;
    fn issue(&self, principal: &PrincipalId, scope: &Scope) -> Result<Attestation, SecError>;
    fn authorize(&self, attestation: &Attestation, action: Action, scope: &Scope)
        -> Result<Decision, SecError>;
    fn revoke(&mut self, principal: &PrincipalId, scope: &Scope) -> Result<(), SecError>;
}
```

### integrity

Purpose: bind a record to its `CoordPath`, its principal, and an epoch so that any modification is detectable.

Operations:

| Operation | Inputs | Outputs | Guarantee |
|-----------|--------|---------|-----------|
| seal | Record, CoordPath, PrincipalId, Epoch | Seal | Seal changes when record, path, principal, or epoch changes |
| verify | Record, CoordPath, Seal | Result | Accepts only seals that match all bound inputs |
| refresh | Seal, Epoch | Seal | Re-scopes an existing seal to a newer epoch without altering the record |

The seal is a keyed commitment. It does not hide the record; it makes modification evident. Replay is limited by the epoch bound.

Informative draft:

```rust
pub trait Integrity {
    fn seal(&self, record: &[u8], path: &CoordPath, principal: &PrincipalId,
            epoch: Epoch) -> Result<Seal, SecError>;
    fn verify(&self, record: &[u8], path: &CoordPath, seal: &Seal)
        -> Result<(), SecError>;
    fn refresh(&self, seal: &Seal, epoch: Epoch) -> Result<Seal, SecError>;
}
```

### audit

Purpose: append-only chained evidence log with inclusion proofs and exportable verification.

Operations:

| Operation | Inputs | Outputs | Guarantee |
|-----------|--------|---------|-----------|
| append | Event | EntryId | Entries are append-only and chained to the previous entry |
| verify_chain | EntryId, EntryId | Result | Chain integrity holds between two entries |
| prove | EntryId | InclusionProof | Proof verifies without the full log |
| export | Range | EvidenceBundle | Bundle can be verified by an external party |

Chaining binds each entry to its predecessor, so retroactive modification of an interior entry breaks the chain. The log carries the evidence needed by the channel module.

Informative draft:

```rust
pub trait Audit {
    fn append(&mut self, event: &Event) -> Result<EntryId, SecError>;
    fn verify_chain(&self, from: EntryId, to: EntryId) -> Result<(), SecError>;
    fn prove(&self, entry: EntryId) -> Result<InclusionProof, SecError>;
    fn export(&self, range: Range<EntryId>) -> Result<EvidenceBundle, SecError>;
}
```

### channel

Purpose: non-repudiation of origin and receipt for evidence exchanged between principals, including coordination nodes.

Operations:

| Operation | Inputs | Outputs | Guarantee |
|-----------|--------|---------|-----------|
| sign | Evidence, SigningKey | SignedEvidence | Signature verifies under the matching verification key |
| verify | SignedEvidence, VerificationKey | Result | Rejects altered evidence or wrong key |
| exchange | LocalEvidence, RemoteNode | Receipt | Receipt proves delivery and origin to a third party |

The channel module produces the non-repudiation property. Receipts are appended to the audit log by the caller. Epoch or nonce values in the evidence prevent replay of signed messages.

Informative draft:

```rust
pub trait Channel {
    fn sign(&self, evidence: &Evidence, key: &SigningKey) -> Result<SignedEvidence, SecError>;
    fn verify(&self, signed: &SignedEvidence, key: &VerificationKey) -> Result<(), SecError>;
    fn exchange(&mut self, local: &Evidence, remote: &NodeId) -> Result<Receipt, SecError>;
}
```

## Delos composite

`Delos` is the composite security engine built on the tagma-sec primitives. It composes the four module implementations, the hybrid confidentiality interface, and the neXus client. The Interpreter role from the initial framing is not part of the composition. A Delos deployment wires the modules to a concrete authority, a seal scheme, an audit store, and a signing key set; the wiring is deployment configuration, and the module interfaces remain the contract.

## Hybrid confidentiality

The structural layer claims no confidentiality property. When confidentiality is required, the payload is encrypted with a hybrid scheme (an ephemeral key combined with an AEAD) before the tagma-sec envelope is applied. The composition order is fixed: encrypt the payload, then seal and sign the envelope. This keeps the integrity and non-repudiation guarantees independent of the encryption scheme and lets the encryption scheme be replaced without changing the module interfaces.

Boundary between fact and conjecture: the ordering rule is a design decision, not a proven result. The rationale is that the seal and the signature bind the ciphertext rather than the plaintext, so a later swap of the encryption scheme does not invalidate existing seals.

Algorithm selection: AEAD choices such as AES-256-GCM or ChaCha20-Poly1305 retain practical strength under quantum adversaries. Signature choices differ: Ed25519 is vulnerable to Shor's algorithm, so post-quantum resistance requires a candidate such as ML-DSA. The final selection is deferred to the Open questions section.

## neXus integration

neXus is the coordination layer of synTagma: topology mapping, routing, and the distributed resolver. The integration contract attaches the four modules to coordination traffic.

- Every coordination message carries an integrity seal over its payload, the target path, the sender principal, and the current epoch.
- Routing table updates require an attestation from the authority module scoped to the affected paths.
- Coordination events are appended to the shared audit log; resolvers verify the chain before trusting a peer's routing state.
- Resolver-to-resolver evidence exchange uses the channel module, and receipts enter the audit log.

The exact neXus message schema is specified by the coordination layer itself; this section fixes only the security envelope that wraps it. Replay of routing updates is prevented by the epoch bound on the seal and by the channel nonce.

## Development plan

The implementation proceeds in four module milestones followed by integration, in dependency order.

| Milestone | Scope | Deliverable |
|-----------|-------|-------------|
| 1 | authority | Registration, attestation, scope authorization, revocation |
| 2 | integrity | Seal scheme with record, path, principal, epoch binding |
| 3 | audit | Append-only chained log with inclusion proofs |
| 4 | channel | Signed evidence exchange with receipts |
| 5 | Integration | neXus envelope, hybrid confidentiality composition, Delos wiring |

The initial framing's five-phase roadmap (tagma-sec primitives, Delos core, neXus, hybrid, applications) maps onto this plan: its first phase splits into milestones 1 to 4, its third and fourth phases correspond to milestone 5, and the application phase remains a downstream activity. Audit anchoring and the epoch source are validated in the audit and integration milestones before wider deployment.

## Compliance

A tagma-sec implementation is compliant iff:

- Each module implements the operations listed in its interface table with the stated guarantees.
- The integrity seal covers the record, the path, the principal, and the epoch.
- The audit log is append-only and every entry chains to its predecessor.
- The channel signature verifies under the published verification key and evidence carries an epoch or nonce.
- Confidentiality is never claimed from coordinate structure; encryption, when present, is applied before sealing per Section `Hybrid confidentiality`.

## Open questions

- Concrete epoch source: a monotonic counter, a trusted time authority, or a hybrid of both.
- Authority key management: self-issued root, hierarchical delegation, or external PKI.
- Seal construction: symmetric MAC, keyed hash, or asymmetric signature per message.
- Audit log topology: single node, replicated chain, or Merkle aggregation across nodes.
- Whether the channel module also covers delivery confirmation or only origin and receipt.
- Signature algorithm family: Ed25519 for speed, a post-quantum candidate such as ML-DSA for quantum resistance, or a hybrid of both.
- Audit anchoring: external transparency log, periodic commitment publication, or standalone chain heads held by independent verifiers.

---

*Version 0.2 draft. Tagma Security Layer Specification (tagma-sec).*
