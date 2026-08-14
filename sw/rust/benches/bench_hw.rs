// ===========================================================================
// Tagma hardware decoder — software reference benchmark
// ===========================================================================
//
// Software baseline for the hardware track (hw/, issue #48). The Verilog
// decoder in hw/rtl/tagma_decoder.v implements the same decomposition as
// Coord::to_axes; these benches measure the reference implementation in
// software so the hardware numbers can be read against a host baseline.
//
// Hardware results (hw/docs/synthesis_report.md, branch 48-hw-verification):
//   generic synthesis       478 cells   (ev stat -json schema, 0 registers)
//   gate-level estimate     588 cells   (2-input library, abc -g)
//   iCE40 synthesis         201 LUT4 + 41 SB_CARRY, 0 registers
//   PnR (UP5K, Upduino 3.1) 254 ICESTORM_LC, 33 SB_IO, Fmax 16.35 MHz
//   worst-case decode       61.17 ns    (critical path, icetime)
//   exhaustive verification 11,172 / 11,172 (RTL and gate netlist, golden)
//   naive shift-subtract    206..232 cells, Fmax 8.63 MHz (no 12 MHz closure)
//
// Software reference (this file, measured):
//   hw/decode/all_11172        1.92 µs   (to_axes over the full space)
//   hw/decode/single           1.44 ns   (one decomposition, no I/O)
//   hw/decode/index_all_11172  0.457 µs  (baseline: raw index extraction)
//   hw/golden/export_11172     4.56 µs   (make golden-export path)
//
// Reading: the worst-case hardware decode (61.17 ns at 16.35 MHz on the
// UP5K) uses a multiply-shift division network; the software reference
// needs about 1.4 ns per decode on the benchmark host. The comparison is
// architectural, not a claim of parity: the hardware number is a post
// place-and-route critical path, the software number is a single-threaded
// host measurement.
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
