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
// The identity is part of that surface. `CoordId` is the same coordinate path read
// as a name, and the record layer builds its addresses from it, so a device without
// a heap has to print, parse, order, and hash it without one. That is exercised
// here rather than assumed, because the identity crossed into this crate from the
// layer above and an allocator would otherwise enter the low layer unnoticed.
//
// On host targets this compiles to a stub, so workspace builds stay green.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

#[cfg(target_arch = "riscv32")]
mod firmware {
    use core::hash::{Hash, Hasher};
    use core::sync::atomic::{AtomicI32, Ordering};

    use tagma_core::{Coord, CoordId, CoordPath};
    use tagma_matrix::{gemv, ColMajor, Elements, Matrix, MatrixRef, RowMajor};

    /// The bytes the wire form takes for a two by three matrix.
    const WIRE: usize = 9 + 2 * 3 * 5;

    /// The identity depth the record layer addresses with.
    const DEPTH: usize = 20;

    /// Weights that live in read-only memory, so the borrowed path is linked too.
    static WEIGHTS: [[i8; 3]; 2] = [[1, 2, 3], [4, 5, 6]];

    /// Where the results land, so none of the work is dead.
    static SINK: AtomicI32 = AtomicI32::new(0);

    /// A stack buffer the canonical text is written into, so printing an identity
    /// is shown not to reach for the heap.
    struct StackText {
        bytes: [u8; DEPTH * 3],
        len: usize,
    }

    impl StackText {
        const fn new() -> Self {
            Self {
                bytes: [0u8; DEPTH * 3],
                len: 0,
            }
        }

        fn as_str(&self) -> Option<&str> {
            core::str::from_utf8(&self.bytes[..self.len]).ok()
        }
    }

    impl core::fmt::Write for StackText {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let end = self.len.checked_add(text.len()).ok_or(core::fmt::Error)?;
            if end > self.bytes.len() {
                return Err(core::fmt::Error);
            }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    /// A hasher small enough to live on the stack, so the identity's `Hash`
    /// surface is linked without pulling anything in.
    struct Fnv(u64);

    impl Default for Fnv {
        fn default() -> Self {
            Self(0xcbf2_9ce4_8422_2325)
        }
    }

    impl Hasher for Fnv {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes {
                self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }

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

        // The identity: a coordinate path built, printed into a stack buffer, read
        // back from that text, ordered, and hashed. Printing, parsing, ordering, and
        // hashing all run here, so a change that reaches for the heap fails the link.
        let zero = Coord::new(0).expect("zero is inside the coordinate space");
        let mut coords = [zero; DEPTH];
        for (axis, slot) in coords.iter_mut().enumerate() {
            if let Some(coord) = Coord::new(axis as u16) {
                *slot = coord;
            }
        }
        let id = CoordId(CoordPath::new(coords));

        let mut text = StackText::new();
        if core::fmt::write(&mut text, format_args!("{}", id)).is_ok() {
            total = total.wrapping_add(id.path().len() as i32);
            total = total.wrapping_add(id.iter().count() as i32);
            if let Some(printed) = text.as_str() {
                match CoordId::<DEPTH>::from_string(printed) {
                    Some(parsed) => {
                        total = total.wrapping_add(if parsed == id { 1 } else { 0 });
                        total = total.wrapping_add(i32::from(parsed.coords()[DEPTH - 1].index()));
                    }
                    None => total = total.wrapping_sub(1),
                }
            }
        }

        let earlier = CoordId(CoordPath::new([zero; DEPTH]));
        total = total.wrapping_add(if earlier < id { 1 } else { 0 });

        let mut hasher = Fnv::default();
        id.hash(&mut hasher);
        total = total.wrapping_add(hasher.finish() as i32);

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
