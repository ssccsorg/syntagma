# Chton store surface and nexus adoption: status report

This report consolidates the state of the chton store surface work (chton issue #6), the nexus adoption (nexus issue #172), the build breakage and fixes applied on 2026-08-09, and the dependency policy change that followed. It closes with a risk assessment and the open decision points.

## The layer boundary and the vision

The agreed layer boundary (2026-08-05) splits the stack into three owners:

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| specification | tagma (syntagma) | CoordSpace family, content as address |
| behavior | chton | all storage and IO operations |
| semantics | nexus | FIH encoding, predicates, swarm runtime |

The strategic intent behind chton is larger than fixing nexus storage. Chton is the common lower IO layer for every spatial-first project in the organization (rem, exaspec, exaverif expected among them), so that upgrades to chton propagate to all consumers automatically. The longer journey, documented in the tagma-signal draft, extends chton from file and digital storage toward the lowest-level infrastructure of physical signal transmission (RF, optical, acoustic). On the commercial track, the pager draft plans a legacy non-spatial KV storage product (Redis RESP and S3 REST adapters, clustering, security controls) built on the same coordinate core.

Current reality against that vision: the file and digital storage layer is complete and adopted by the first consumer (nexus). The signal layer exists only as a design document. The commercial product is at the planning stage. The other consumer projects have not started adoption.

## Delivery status

| Item | Scope | Implementation | Verification | State |
|------|-------|----------------|--------------|-------|
| chton #6 | store surface in chton: EntityStore family, Cell2, CoordMapStoreIo | merged to chton main (`e3ec990`): `store`, `cell`, `io` modules | chton `run.sh --check` green, 11 tests, wasm32 clean | issue open |
| nexus #172 phase 1 | CoordMapStore and CoordMapStoreIo adoption, drop local CoordMapIo | merged to nexus main (`c14e8b22`, `0218797c`) | green | issue open |
| nexus #172 phase 2 | EntityStore family absorbed, re-export from chton, drop tagma-map from nex-fih | merged to nexus main (`aa9794fb`), 9 files, +46/-835, `entity_store.rs` removed | 225 tests, clippy `-D warnings`, wasm32, fmt, `run.sh` exit 0 | issue open |

Both issues remain open even though the work is on main. Neither went through a pull request; the commits were pushed directly to main.

## The build breakage and its root causes

Running `bash run.sh` on 2026-08-09 exposed two independent failures.

### Stale lockfiles after the refactor

The workspace lockfiles for `nex/`, `apps/nex-api`, `apps/nex-calc-fihcontract`, and `apps/nex-spinwasi-ssccsdocs` pinned chton at `4bf4d72`, a commit that predates the `cell` and `store` modules. The nexus source had already been migrated to the new chton surface (`chton::cell`, `chton::store`, `chton::map::CoordMapStore`, `chton::io::CoordMapStoreIo`), so the wasm check of `nex-fih` and the gateway build of `nex-api` failed to compile. The root lockfile already pinned the newer chton (`e3ec990`), which is why the native workspace check passed while the sub-workspaces failed.

The fix was to run `cargo update -p chton -p tagma-core -p tagma-geo -p tagma-map` in every affected workspace so all lockfiles agree on chton `e3ec990` and syntagma `6b5e448`.

### The verify_nexd socket race

`verify_nexd` in `run.sh` pre-started a nex-server on the test socket, then started nexd with `NEXD_NEX_SERVER_PATH` set, which made nexd spawn a second nex-server as a managed child on the same socket path. The child removes the stale socket file at startup and re-binds after initialization. During that window the socket path does not exist, and the first JSON-RPC request failed with `No such file or directory` (ENOENT). The failure was timing-dependent: it appeared when the first RPC arrived about 30 ms after the child was spawned, before the child finished binding.

The pre-start was added deliberately in `7e3887da` as a readiness workaround, and the `NEX_DATA_DIR` inheritance fix in `6f0c663c` covered the test harness but never reached `run.sh`. The nexd design (documented in its README, `main.rs`, and the integration tests) is that nexd spawns and supervises nex-server itself.

The fix removed the pre-start entirely and passed `NEX_DATA_DIR` to nexd so the managed child keeps state in the temp directory. The seven verification scenarios are unchanged and all pass. The standalone nex-server coverage remains in `verify_nex_server`, and readiness checking is now stronger because nexd refuses to start its IPC server until it can connect to its managed nex-server.

## Dependency policy change

The commit `7e9978b8` (local nexus main) untracked all eight `Cargo.lock` files and added `Cargo.lock` to `.gitignore`. The rationale is that Cargo records the resolved commit for git dependencies inside the lockfile, and a committed lockfile therefore pins chton and syntagma to specific commits. Untracking the lockfiles makes fresh builds resolve both git dependencies to the head of their declared branch (`main`), which is the mechanism behind the automatic-propagation goal.

This is a deliberate trade: reproducibility is exchanged for always tracking branch heads. Local builds still use the on-disk lockfile until `cargo update` is run, so the current pins (chton `e3ec990`, syntagma `6b5e448`) remain in effect locally.

## The str_to_coordpath contract change

The nexus string-to-coordinate mapping changed semantics during adoption. The previous ByteWise mapping converted each byte to a Coord, truncating keys longer than N bytes and zero-padding shorter keys. That created a structural collision class (`"ab"` versus `"ab\0"`, and truncated pairs at the N-byte boundary). The chton mapping uses a SHA-256 fingerprint split into N Coord values with modular folding, with no truncation and no padding, and a per-key-pair collision probability around 2^-80 for N=6.

The decision was to keep SHA-256 (option A). Two notes qualify this decision. FIH addressing does not go through `str_to_coordpath`; the 19-axis store encodes the `FihHash` directly via `encode_hash_into_axes`, so the mapping only affects string-keyed bridge storage. And option B (an absolutely injective length-prefixed encoding) would impose a key length limit, which is why it was not adopted.

## Residual items

- `nex/process/tests/coord_map_proximity.rs` (phase 2 scenario from issue #166) still imports `tagma_map::CoordMap` and `CoordCubeMap` as a dev-dependency. The issue #172 verification criterion "nexus no longer depends on tagma-map" is therefore not strictly satisfied, although the library code is clean.
- Issues #172 and #6 are open with no pull request trail, so the review history for the shared foundation changes is absent.
- Data written under the ByteWise mapping is not readable under the SHA-256 mapping. The pre-1.0 status makes the impact negligible, but no migration plan exists.

## Risk assessment

- Floating git dependencies mean a breaking change on chton main breaks every consumer at the same time. The consumer base is currently a single project (nexus), which keeps the blast radius small, but the risk grows with each new adopter.
- Direct-to-main commits on the shared foundation crate bypass review. Both issue #6 and issue #172 work entered main this way.
- Consumer adoption is at zero for rem, es (exaspec), and ev (exaverif); a grep for chton references in those repositories returns nothing.
- The distance from the current code to the signal layer and to the commercial product is large. The nearest commercial output is the CoordMapStore plus FileIo surface already present.

## Recommendations

- Close nexus #172 after deciding the disposition of `coord_map_proximity.rs`: keep it with a documented dev-dependency exception, or drop it since chton already carries a spatial proximity test (`search_json.rs`).
- Record the SHA-256 mapping contract (no truncation, no padding, collision probability) on chton issue #6 before closing it.
- Confirm that chton main enforces its `run.sh` gate before pushes. If not, at least keep the issue-to-commit link and verification record visible.
- Treat the signal layer and the commercial KV product as separate tracks. The commercial KV is the nearer output; the signal layer follows the tagma-signal plan.
- Start consumer adoption with the project whose spatial computation surface is already identified, then expand.

## References

- nexus issue #172: move nexus storage behavior onto chton surfaces, <https://github.com/ssccsorg/nexus/issues/172>
- chton issue #6: add store surface (EntityStore family) to chton, <https://github.com/ssccsorg/chton/issues/6>
- nexus main: `aa9794fb` (phase 2), `7e9978b8` (lockfile policy and run.sh fixes)
- chton main: `e3ec990` (store surface)
- tagma-signal draft: <file:///Users/blackgene/Documents/ssccs/docs/projects/syntagma/tagma/signal/index.qmd>
- chton commercial draft: <file:///Users/blackgene/Documents/ct/docs/pager.qmd>
