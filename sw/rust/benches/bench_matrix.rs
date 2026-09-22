// Tagma-matrix: what coordinate addressing costs against a flat loop, and what the
// product and the rescale cost at the size of a layer.
//
// The comparison that matters is `matrix/gemv_addressed` against `matrix/gemv_flat`.
// The flat loop is what an engine that addresses elements by offset writes, so the
// gap between the two is the price of an element's address being its coordinate.
//
// Baseline, recorded so that a later run can tell a regression from noise and so that
// the numbers quoted in the documentation have a source. The host is an Apple M1 Max
// on macOS 15.7.1 with rustc 1.95.0, and each entry is the median of 30 samples in the
// release profile. These are a reference point for this suite on this host, and no
// claim about any target:
//
//   matrix/gemv_addressed                 7.32 us   (64 x 256)
//   matrix/gemv_flat                      1.82 us   (64 x 256)
//   matrix/gemv_requantized               7.36 us   (64 x 256)
//   matrix/wire_encode                   16.26 us   (64 x 256)
//   matrix/matmul_64x64x64               38.18 us
//   matrix/matmul_requantized_64x64x64   37.73 us
//   matrix/requantize_256               194.8  ns
//
// The addressed product is about four times the flat one. The rescale adds nothing
// measurable to the product, since it is one multiply and one shift against 256 of
// them, which is why `gemv_requantized` and `gemv_addressed` sit on top of each other.
// The contraction is dense, so its cost is the cube of its side rather than a row.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tagma_matrix::{
    gemv, gemv_requantized, matmul, matmul_requantized, requantize_into, Elements, Matrix, Requant,
};

/// The logical value at row `i`, column `j`, with a signed spread that keeps the
/// accumulation from cancelling out.
fn value_at(i: usize, j: usize) -> i8 {
    ((i as i32 * 7 + j as i32 * 13) % 61 - 30) as i8
}

fn row_major_buffer<const R: usize, const C: usize>() -> [[i8; C]; R] {
    let mut data = [[0i8; C]; R];
    for (i, row) in data.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = value_at(i, j);
        }
    }
    data
}

fn activation<const C: usize>() -> [i8; C] {
    let mut x = [0i8; C];
    for (j, cell) in x.iter_mut().enumerate() {
        *cell = ((j as i32 * 11) % 41 - 20) as i8;
    }
    x
}

const ROWS: usize = 64;
const COLS: usize = 256;
const SIDE: usize = 64;

/// The same product over a flat buffer, addressing each element by its offset. An
/// engine without coordinates writes this loop.
fn flat_gemv(a: &[i8], x: &[i8], y: &mut [i32]) {
    let cols = x.len();
    for (i, out) in y.iter_mut().enumerate() {
        let mut accumulator = 0i32;
        for (j, activation) in x.iter().enumerate() {
            accumulator += i32::from(a[i * cols + j]) * i32::from(*activation);
        }
        *out = accumulator;
    }
}

fn product(c: &mut Criterion) {
    let matrix = Matrix::<ROWS, COLS>::new(row_major_buffer::<ROWS, COLS>());
    let flat: Vec<i8> = (0..ROWS)
        .flat_map(|i| (0..COLS).map(move |j| value_at(i, j)))
        .collect();
    let x = activation::<COLS>();
    let rescale = Requant::new(38_997_123, 27);

    c.bench_function("matrix/gemv_addressed", |b| {
        b.iter(|| {
            let mut y = [0i32; ROWS];
            gemv(black_box(&matrix), black_box(&x), &mut y);
            black_box(y)
        })
    });

    c.bench_function("matrix/gemv_flat", |b| {
        b.iter(|| {
            let mut y = [0i32; ROWS];
            flat_gemv(black_box(&flat), black_box(&x), &mut y);
            black_box(y)
        })
    });

    c.bench_function("matrix/gemv_requantized", |b| {
        b.iter(|| {
            let mut y = [0i8; ROWS];
            gemv_requantized(
                black_box(&matrix),
                black_box(&x),
                black_box(&rescale),
                &mut y,
            );
            black_box(y)
        })
    });

    c.bench_function("matrix/wire_encode", |b| {
        let mut bytes = [0u8; 9 + ROWS * COLS * 5];
        b.iter(|| {
            black_box(
                matrix
                    .encode_into(black_box(&mut bytes))
                    .expect("the buffer holds the matrix"),
            )
        })
    });
}

fn contraction(c: &mut Criterion) {
    let left = Matrix::<SIDE, SIDE>::new(row_major_buffer::<SIDE, SIDE>());
    let right = Matrix::<SIDE, SIDE>::new(row_major_buffer::<SIDE, SIDE>());
    let rescale = Requant::new(38_997_123, 27);

    c.bench_function("matrix/matmul_64x64x64", |b| {
        b.iter(|| {
            let mut out = [[0i32; SIDE]; SIDE];
            matmul(black_box(&left), black_box(&right), &mut out);
            black_box(out)
        })
    });

    c.bench_function("matrix/matmul_requantized_64x64x64", |b| {
        b.iter(|| {
            let mut out = Matrix::<SIDE, SIDE>::new([[0i8; SIDE]; SIDE]);
            matmul_requantized(
                black_box(&left),
                black_box(&right),
                black_box(&rescale),
                &mut out,
            );
            black_box(out)
        })
    });
}

fn rescale(c: &mut Criterion) {
    let rescale = Requant::new(38_997_123, 27);
    let accumulators: Vec<i32> = (0..COLS as i32).map(|i| (i - 128) * 977).collect();
    let mut elements = [0i8; COLS];

    c.bench_function("matrix/requantize_256", |b| {
        b.iter(|| {
            requantize_into(black_box(&accumulators), black_box(&rescale), &mut elements);
            black_box(elements)
        })
    });
}

criterion_group!(benches, product, contraction, rescale);
criterion_main!(benches);
