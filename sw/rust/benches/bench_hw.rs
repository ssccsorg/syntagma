// ===========================================================================
// Tagma hardware decoder: software reference benchmark
// ===========================================================================
//
// Software baseline for the hardware track (hw/, issue #48). The Verilog
// decoder in hw/rtl/tagma_decoder.v implements the same decomposition as
// Coord::to_axes; these benches measure the reference implementation in
// software so the hardware numbers can be read against a host baseline.
//
// Core results (measured; full report in
// docs/devlogs/hw/2026-08-14-synthesis-report.md, branch 48-hw-verification):
//
// Verification:
//   exhaustive RTL simulation    11,172 / 11,172  (formula mode)
//   golden anchors               11,172 / 11,172  (Rust reference mode)
//   gate-level netlist sim       11,172 / 11,172
//   formal equivalence RTL/net   50 cells proven, all 2^16 inputs
//
// Synthesis:
//   generic (ev stat -json)      478 cells, 0 registers
//   gate-level estimate          588 cells (2-input library, abc -g)
//   iCE40                        201 LUT4 + 41 SB_CARRY, 0 registers
//
// Design trade-off (naive vs multiply-shift):
//   shift-subtract (naive)       206..232 cells, Fmax 8.63 MHz, 115.90 ns
//   multiply-shift (demo)        478..588 cells, Fmax 16.79 MHz, 59.55 ns
//   the ~300 gate claim holds for the naive structure; timing closure on
//   a 12 MHz board costs about 2.5x gates
//
// PnR on UP5K (Upduino 3.1, 12 MHz board clock):
//   ICESTORM_LC                  255 / 5280 (4%)
//   SB_IO                        33 / 39 (84%)
//   critical path                59.55 ns
//   Fmax                         16.79 MHz, 12 MHz closes with margin
//   logic levels                 33
//
// Software reference (this file, measured):
//   hw/decode/all_11172        1.92 µs   (to_axes over the full space)
//   hw/decode/single           1.44 ns   (one decomposition, no I/O)
//   hw/decode/index_all_11172  0.457 µs  (baseline: raw index extraction)
//   hw/golden/export_11172     4.56 µs   (make golden-export path)
//
// Reading: the worst-case hardware decode (59.55 ns at 16.79 MHz on the
// UP5K) uses a multiply-shift division network; the software reference
// needs about 1.4 ns per decode on the benchmark host. The comparison is
// architectural, not a claim of parity: the hardware number is a post
// place-and-route critical path, the software number is a single-threaded
// host measurement. The hardware value is energy, determinism, and
// physical embedding, not per-op speed: the whole 11,172-entry space fits
// in L1 cache, which the software already exploits at 0.38 ns per access.
// ===========================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tagma_core::Coord;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const N: usize = Coord::N_VALID;

/// N pre-computed valid coordinates.
fn all_coords() -> Vec<Coord> {
    (0..N as u16).map(|i| Coord::new(i).unwrap()).collect()
}

// ===========================================================================
// Decoder microbenchmarks (software reference)
// ===========================================================================

// hw/decode/all_11172         1.92 µs
fn bench_hw_decode_all(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("hw/decode/all_11172", |b| {
        b.iter(|| {
            let mut s = 0u8;
            for &coord in &coords {
                let (i, m, f) = coord.to_axes();
                s = s.wrapping_add(i).wrapping_add(m).wrapping_add(f);
            }
            black_box(s)
        })
    });
}

// hw/decode/single            1.44 ns
fn bench_hw_decode_single(c: &mut Criterion) {
    let coord = Coord::new(0x2BA3).unwrap(); // last syllable, offset 11171
    c.bench_function("hw/decode/single", |b| {
        b.iter(|| {
            let (i, m, f) = coord.to_axes();
            black_box((i, m, f))
        })
    });
}

// hw/decode/index_all_11172   0.457 µs  (baseline without decomposition)
fn bench_hw_decode_index_all(c: &mut Criterion) {
    let coords = all_coords();
    c.bench_function("hw/decode/index_all_11172", |b| {
        b.iter(|| {
            let mut s = 0u32;
            for &coord in &coords {
                s = s.wrapping_add(coord.index() as u32);
            }
            black_box(s)
        })
    });
}

// hw/golden/export_11172      4.56 µs   (packed anchors, make golden-export)
fn bench_hw_golden_export(c: &mut Criterion) {
    c.bench_function("hw/golden/export_11172", |b| {
        b.iter(|| {
            let mut n = 0u32;
            for offset in 0..N as u16 {
                let coord = Coord::new(offset).unwrap();
                let (i, m, f) = coord.to_axes();
                let packed =
                    ((offset as u32) << 15) | ((i as u32) << 10) | ((m as u32) << 5) | (f as u32);
                n = n.wrapping_add(packed);
            }
            black_box(n)
        })
    });
}

criterion_group!(
    name = hw;
    config = Criterion::default().sample_size(100);
    targets = bench_hw_decode_all, bench_hw_decode_single,
              bench_hw_decode_index_all, bench_hw_golden_export
);

criterion_main!(hw);
