# DuckDB coordinate lattice: the measured applicability boundary of the tagma principle

The DuckDB coordinate lattice track applied the tagma coordinate-lattice
principle to multi-dimensional point and range queries in a real analytical
engine. The measured outcome was partly positive and partly negative, and
the negative results sharpened the applicability conditions of the
principle more than the positive ones. This devlog records the full
technical record, the corrections to the earlier interpretation, and the
strategic conclusion: the tagma win is largest where the incumbent path is
structurally unprunable or I/O-dominated, and DuckDB satisfies neither
condition.

## The track and its measured results

### Phase 1: profile gate (ssccsorg/syntagma #54)

One table of 9,938,375 rows (215 cubed), one row per cell, dimensions
uniform in `[0, 215)`, rows in random order, measured on DuckDB v1.5.5 with
10 threads. The gate confirmed the multi-filter FIXME in `TableScanInitGlobal`
at scale:

- Every multi-filter query (P2, P3, R2, R3) scanned all 9,938,375 rows even
  with indexes on all three dimensions.
- The benchmark point P3 (`d1 = x AND d2 = y AND d3 = z`): 9,938,375 rows
  scanned, 1 matched, 1.35 ms (count) and 1.43 ms (payload).
- The ART index never activated under the default settings: the minimum
  single-dimension selectivity (0.465 percent) exceeds the 0.1 percent
  threshold. Forced to 1.0, the ART path took 25.3 ms against 1.27 ms for
  the full scan.
- 100,000 point queries: 136.5 s (1.37 ms per query) through the python
  driver.

### Phase 2: lattice prototype and the scaling law

The physical layout is lattice-sorted storage. The `lattice_scan` table
function maps a cell to a row offset with the closed form
`n = ((d1 * E2 + d2) * E3 + d3)` and enumerates a range box with a
mixed-radix odometer. Correctness is verified against the regular scan
(`verify_lattice.sh`).

The A/B measured the per-query point cost across four table sizes with the
release build:

| Table | scan per query | lattice per query | ratio |
| :--- | ---: | ---: | ---: |
| 9.9M | 0.479 ms | 0.311 ms | 1.54x |
| 100M | 1.091 ms | 0.312 ms | 3.5x |
| 300M | 2.203 ms | 0.325 ms | 6.8x |
| 600M | 4.106 ms | 0.362 ms | 11.3x |

The lattice per-query cost is flat in the table size; the scan grows with
N; the ratio grows with N. This was recorded as the measured verification
of the ExaVerif pattern in a database engine.

### Phase 3 refinement: the join layer (ssccsorg/syntagma #55)

The perfect hash join for dense bounded single keys was generalized to D
equality conditions with a mixed-radix fold. The implementation is correct
(the existing PHJ suite and `verify_phj.sh` pass), and the A/B shows a
limited advantage:

| Workload | PHJ | regular HT | ratio |
| :--- | ---: | ---: | ---: |
| 3D 90^3 build-dominated self join | 20 ms | 16 ms | 0.8x (loss) |
| probe 1k build by 1M probe 3D | 3 ms | 4 ms | 1.33x |
| probe 1k build by 1M probe 2D | 2 ms | 2 ms | 1.0x |

The PHJ build scans D key columns and gathers into a position table, which
is structurally more work than the vectorized hash build for a single pass.
The probe-side O(1) position lookup is where the gain comes from, and the
gain is small because the regular hash probe is already near-O(1).

### Cold-cache I/O: refuted

The cold-cache measurement purged the OS page cache with root `purge` plus
24 GB of memory pressure before every query, three samples, on the 600M
table (2.56 GB file):

| Query | cold scan | cold lattice | cold ratio |
| :--- | ---: | ---: | ---: |
| P3 point | 27 ms | 10 ms | 2.7x |
| R3 range | 27 ms | 15 ms | 1.8x |
| S1 wide | 26 ms | 873 ms | lattice loses |

The cold ratio is smaller than the warm ratio (16x at the CLI timer, 11.3x
in-process). The scan is zonemap-pruned to 2 of 4,876 row groups, so its
cold read is about 20 MB of row-group metadata plus residual data, not the
whole file. The lattice pays a fixed cold-start latency of 9 to 10 ms,
independent of the table size. The ROOT-style I/O dominance does not
transfer to this workload, because the sorted layout prevents the scan from
ever being I/O-dominated.

### Sparse boundary: the density requirement

The closed form addresses a row offset only when every cell is present. At
10 percent density (600M cells, 59.9M rows) the point query falls back to a
stock single-column index on the cell:

| Measure | dense 600M | sparse 600M (10 percent) |
| :--- | ---: | ---: |
| data file | 2.56 GB | 128 MB |
| map or index | none needed | cell ART index +1.54 GB |
| point scan | 16 ms | 2.0 ms |
| lattice or index path | 1.0 ms (lattice) | 2.0 ms (ART) |

Sparsity weakens the incumbent 8x (fewer row groups), and the index path is
at parity with the sparse scan (2.0 against 2.0 ms), against the dense
11.3x. The dense cell map compresses well in DuckDB's columnar format
(sparse data plus map at 26 MB against the dense 32 MB at the 215 space),
smaller than the raw 80 MB estimate in the plan, but the ART index over the
map costs 79 MB at 215 and 1.54 GB at 600M.

## The corrections to the record

The cold-cache experiment exposed a hidden assumption in the earlier A/B
interpretation, and the record was corrected in the ssccs documents:

- The A/B scan baseline is a zonemap-pruned scan, not a full table scan.
  EXPLAIN ANALYZE on the 600M table reports 2 of 4,876 row groups scanned
  for the P3 point.
- The scan cost grows with N through the per-row-group metadata traversal
  (about 0.76 us per row group), not through the data volume.
- The full table scan (9,938,375 rows, 1 matched) was measured only on the
  random-order gate table; the lattice-sorted A/B tables prune.
- The win mechanism is therefore skipping the row-group metadata traversal
  plus the residual row-group reads, not avoiding an N-row scan.

The gate problem and the lattice solution were demonstrated on different
layouts: the problem (full scan) on random order, the solution on sorted
order, where the problem is partially gone. The corrected record separates
the two explicitly.

## What the results say about the tagma winning conditions

| Condition | DuckDB | where tagma wins |
| :--- | :--- | :--- |
| pruning possibility of the incumbent | zonemap prunes 4,876 to 2 row groups | structurally unprunable or expensive structures (ExaVerif full space, ROOT index traversal) |
| cold-cache I/O dominance | 2.7x, not dominant | I/O-dominated paths (ROOT, 69 to 181x) |
| query complexity | simple point and range on 1 to 3 axes | multi-dimension compound conditions with selective filters |
| data structure | sorted table that enables zonemaps | random or unordered layouts where index traversal is required |
| density | dense lattice required for the closed form | bounded enumerable dense lattices |

The win is largest when the incumbent is structurally unprunable or when
I/O and traversal dominate. DuckDB satisfies neither condition, so the win
converged to a constant factor.

## Domain re-assessment

| Domain | fitness | basis |
| :--- | :--- | :--- |
| ExaVerif (RISC-V verification) | high | full space to valid subspace, unprunable |
| CERN ROOT (HEP I/O) | high | I/O-dominated, per-read index traversal |
| DuckDB (analytical DB) | low (tagma principle) | zonemap pruning shrinks the tagma win to a constant factor; the track still produced a measured 3-5x standard-engineering contribution, the multi-filter index scan |
| vLLM and llama.cpp (KV cache) | moderate | index traversal exists but compute dominates, parity |
| vector DB (LanceDB and similar) | moderate to high | ANN search is traversal-dominated, unprunable |

## The direction from the DuckDB lesson

1. Selective queries alone are insufficient. DuckDB's zonemaps already
   prune most selective queries; the lattice must beat the pruned path, not
   a full scan.
2. The tagma win requires a domain where pruning is structurally
   impossible: ExaVerif (full space), ROOT (index traversal), ANN
   traversal in vector search.
3. Cold-cache I/O is not a universal bottleneck remedy. The DuckDB cold win
   was 2.7x.
4. The data structure is decisive. Sorted tables enable cheap pruning that
   competes with the closed form; random or unordered layouts expose the
   traversal cost the closed form removes.

The options are: continue DuckDB engineering (constant-factor wins, now duplicated upstream by duckdb/duckdb pull request #24942, so the value is the measurement record), exit the DuckDB track (lessons recorded, results retained as assets that define the applicability conditions), or move to unprunable and traversal-dominated domains (vector databases, Iceberg metadata, Arrow metadata parsing). The recommendation is the third: DuckDB was a valuable experiment, and its result refines the strategic direction of syntagma rather than refuting the principle.

## The multi-filter index scan result (ssccsorg/syntagma #56)

A follow-up track implemented the TableScanInitGlobal FIXME on the fork
branch `56-duckdb-multi-filter-index`: for multi-filter point queries, scan
one single-column ART per filtered column and intersect the row-ID sets. The
implementation also fixed a latent binding bug in `TryScanIndex` (the
unbound index expression references the indexed columns positionally, and
the update condition compared that positional index against the table column
id, so index scans on non-first columns never activated).

Measured on a 100M-row random-order 2D table where the scan cannot prune
(extent 10,000, per-dimension selectivity 0.01 percent, below the 0.1
percent index-scan threshold):

| Query | scan | multi-filter index | ratio |
| :--- | ---: | ---: | ---: |
| count, warm | 37 ms | 10 ms | 3.7x |
| payload, warm | 34 ms | 7 ms | 4.9x |
| count, cold | 102 ms | 26 ms | 3.9x |
| payload, cold | 98 ms | 22 ms | 4.5x |

The first cold sample measures 224 ms against 26 ms (8.6x); the cold ratio
is bounded by the compressed data volume (about 0.7 GB, which fits in RAM).
The scan side is measured with the index scan forced off on the same table,
so both sides run the identical query. Correctness is verified by
`benchmark/lattice/verify_multi_index.sh` (2-filter and 3-filter
intersections, filter order, empty results, missing-index fallback,
threshold non-activation, single-filter regression).

Boundaries: the path activates only when every filtered column carries a
single-column ART and each individual filter stays below the threshold;
range filters fall back to the scan (the pre-existing single-filter
behavior); the two ART indexes dominate the file size (about 2.5 GB each on
100M rows).

Upstream discovery: duckdb/duckdb pull request #24942 implements the same
feature with a more complete design (one-sided ranges, IN filters,
physical-versus-logical column handling, empty-intersection early exit,
sqllogictest coverage) and includes the same binding fix. A duplicate
upstream pull request is therefore not appropriate; the fork work stands as
the measurement record. The upstream PR has no performance measurement, so
the measured 3-5x data is the record's contribution.

Meaning for the record: this is a measured 3-5x DuckDB contribution produced
by the track's measurement discipline and recorded through the syntagma
infrastructure (fork bench scripts, results, this devlog), independent of
upstream adoption. It revises the DuckDB row of the domain table: as a
tagma-principle target the fitness remains low, while as a standard
engineering target the track produced a measured 3-5x result.

## Track outcome

The DuckDB track is closed with the following consolidated record. The
lattice principle is verified in a database engine, but only in the narrow
regime the measurements define: dense bounded integer lattices, warm cache,
point and small range queries, against the zonemap-pruned scan. The cold-cache
I/O dominance claim is refuted, the sparse boundary shows the density
requirement, and the join generalization is correct with a limited measured
advantage. The multi-filter index scan follow-up produced a measured 3-5x
result that upstream independently implements more completely (duckdb/duckdb
pull request #24942), so the fork contribution is the measurement record.
The strategic direction is the unprunable and traversal-dominated domains,
with a profile gate before any new target: bounded enumerable lattice,
dominant cost share, and no cheap incumbent pruning.

## References

- The DuckDB lattice development plan: `/works/duckdb/lattice/` in ssccs
- The DuckDB lattice topic: `/works/duckdb/` in ssccs
- The syntagma core upgrade living note: `/works/duckdb/lattice/syntagma/` in ssccs
- The fork branches `54-duckdb-lattice`, `55-join-probe`, and `56-duckdb-multi-filter-index` in `ssccsorg/duckdb`
- The bench scripts and results under `benchmark/lattice/` on the fork branches
