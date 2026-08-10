# ExaVerif: structural enumeration as a Tagma case study

ExaVerif (ev) is the first production application built on Tagma's
`DynCoordSpace<CoordSet>`. It verifies RISC-V custom instruction encodings
by generating every valid combination of instruction fields and evaluating
each combination against a constraint set. Before Tagma, it did this by
computing the full Cartesian product of field domains (33,554,432
combinations for the CVA6 CV-X-IF space) and filtering each combination
through constraint checks. After Tagma, it encodes the constraints into
a DynCoordSpace and generates only the valid subset (229,376 combinations).

The speedup factor (950x at the largest fixture) is a direct consequence of
Tagma's core thesis: structural addressing eliminates the work of visiting
invalid space. Every combination that the structural pipeline visits, it
evaluates with the same constraint checks. The 99.3% of the space that is
invalid is never allocated, never iterated, and never tested.

## What Tagma provided

The CVA6 encoding space has five fields (funct3, funct7, rs1, rs2, rd) and
three types of constraints: `oneof`, `cross`, and `enable_mask`. The `cross`
constraint maps funct3 values to allowed funct7 ranges --- a parent-to-child
restriction that changes per-parent. Tagma's `DynCoordSpace<CoordSet>`
expresses this mapping as a coordinate space: each parent coord (funct3
value) maps to a CoordSet (allowed funct7 values). The lookup is a single
bit test on a 175-word bit array.

Before Tagma, the cross constraint was implemented as a `HashMap<i64,
Vec<i64>>` checked at runtime for every combination. The HashMap lookup
(compute hash, probe bucket, compare key) cost CPU cycles for every one of
the 33.5M combinations, including the 99.3% that would ultimately be
rejected. After Tagma, the constraint is compiled into the enumeration
structure at setup time. The iterator simply does not visit combinations
where funct3=1 and funct7=64, because the CoordSet for funct3=1 contains
only {0}. No lookup is performed at enumeration time.

## What the numbers show

The structural pipeline is not faster at evaluating constraints. It
evaluates exactly the same constraints on exactly the same combinations.
The difference is what it generates. The standard pipeline generates the
full Cartesian product and then filters. The structural pipeline generates
only the structurally valid subset by encoding constraints into a
DynCoordSpace during setup. Filtering becomes generation.

The benchmark table from the ev white paper:

| Fixture | Raw space | Valid space | Density | Standard evaluate | Structural verify | Speedup |
|---------|:---------:|:-----------:|:-------:|:-----------------:|:-----------------:|:-------:|
| Ibex R-type | 524,288 | 92,160 | 17.6% | 3.66 s | 46.1 ms | 79x |
| CVA6 R4 | 2,097,152 | 12,288 | 0.6% | 1.23 s | 1.60 ms | 766x |
| CVA6 full | 33,554,432 | 229,376 | 0.7% | 29.7 s | 31.3 ms | 950x |

The speedup is proportional to space density: sparser valid spaces yield
larger speedups. This is consistent with the O(N) vs O(V) analysis in the
white paper. At 0.6% density, 766x. At 17.6% density, 79x. At 100% density
(no constraints), 1.2x --- the overhead of Tagma's enumeration setup is
recovered in under 3 combinations.

## What the case study validates

1. **DynCoordSpace<CoordSet> is a correct abstraction for cross constraints.**
   The parent-to-child mapping pattern appears verbatim in the RISC-V spec.
   One DynCoordSpace per constrained field, one CoordSet per parent value.

2. **Structural enumeration eliminates runtime constraint evaluation for
   structurally-enforceable constraints.** The enum iterator visits only
   valid combinations. Runtime constraint checks for non-structural
   constraints (eq, neq, lt, gt, le, ge) are still needed, but these are
   typically a small fraction of the total constraint set.

3. **The setup cost is negligible.** Building the DynCoordSpace for CVA6
   (3 mapping entries, 6 total coord-value pairs) takes under 1 us. The
   benefit is realized within the first 3 combinations.

4. **The iterator is memory-safe.** StructuralEnum allocates no heap memory
   during iteration. The DynCoordSpace is built once during setup. The loop
   body is a lightweight advance-and-check on an array of indices.
