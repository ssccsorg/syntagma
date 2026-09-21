// The link check: the family's no-allocator member, linked into a program with no
// operating system and no global allocator.
//
// A successful link is the proof. If anything underneath reaches for the
// allocator, the link fails with `no global memory allocator found but one is
// required`, so the property is a fact about the artifact rather than a claim in a
// comment.
//
// The body reaches the whole surface the crate exposes, so an addition that
// allocates is reached here rather than left unused and unnoticed. The results are
// stored through an atomic rather than dropped, so none of the work the link
// depends on can be folded away.
//
// On host targets this compiles to a stub, so workspace builds stay green.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

#[cfg(target_arch = "riscv32")]
mod firmware {
    use core::sync::atomic::{AtomicI32, Ordering};

    use tagma_matrix::{gemv, ColMajor, Elements, Matrix, MatrixRef, RowMajor};

    /// The bytes the wire form takes for a two by three matrix.
    const WIRE: usize = 9 + 2 * 3 * 5;

    /// Weights that live in read-only memory, so the borrowed path is linked too.
    static WEIGHTS: [[i8; 3]; 2] = [[1, 2, 3], [4, 5, 6]];

    /// Where the results land, so none of the work is dead.
    static SINK: AtomicI32 = AtomicI32::new(0);

    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn _start() -> ! {
        let activation = [1i8, 2, 3];
        let mut total = 0i32;

        // An owned matrix: construction, a write, the product, and the address
        // surface.
        let mut owned = Matrix::<2, 3, RowMajor>::new([[0i8; 3]; 2]);
        owned.set(0, 0, 1);
        total = total.wrapping_add(i32::from(owned.get(0, 0)));

        let mut product = [0i32; 2];
        gemv(&owned, &activation, &mut product);
        total = total.wrapping_add(product[0]);

        if let Some(path) = owned.path_of(1, 2) {
            if let Some(value) = owned.at(path) {
                total = total.wrapping_add(i32::from(value));
            }
        }

        // The wire form, out and back into another physical order.
        let mut stream = [0u8; WIRE];
        if owned.encode_into(&mut stream).is_ok() {
            total = total.wrapping_add(Matrix::<2, 3, RowMajor>::encoded_len() as i32);
            if let Ok(relocated) = Matrix::<2, 3, ColMajor>::decode(&stream) {
                total = total.wrapping_add(i32::from(relocated.get(0, 0)));
            }
        }

        // A borrowed matrix, for weights that live in read-only memory.
        let borrowed = MatrixRef::<2, 3, RowMajor>::new(&WEIGHTS);
        let mut borrowed_product = [0i32; 2];
        gemv(&borrowed, &activation, &mut borrowed_product);
        total = total.wrapping_add(borrowed_product[1]);

        SINK.store(total, Ordering::Relaxed);

        loop {
            core::hint::spin_loop();
        }
    }
}

#[cfg(not(target_arch = "riscv32"))]
fn main() {
    println!(
        "tagma-linkcheck: the check is the link for a target with no operating system;\n\
         build it with --target riscv32imac-unknown-none-elf"
    );
}
