//! Exports golden anchor vectors from the tagma_core reference engine.
//!
//! The output file holds one packed 29-bit value per line, eight hex
//! digits, for every valid coordinate:
//!
//!   offset[28:15] initial[14:10] medial[9:5] final[4:0]
//!
//! where offset is the 0-based coordinate index in [0, 11171] and the
//! three axes are the choseong, jungseong, and jongseong indices. The
//! file feeds the RTL testbench in hw/rtl (golden mode), so the Verilog
//! decoder is checked against the reference implementation rather than
//! against a second copy of the decomposition formula.
//!
//! Usage:
//!   cargo run -p tagma-core --example golden_export -- <output path>

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use tagma_core::Coord;

fn main() {
    let out = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("hw/rtl/golden_anchors.hex"));

    let file = File::create(&out).expect("failed to create output file");
    let mut writer = BufWriter::new(file);

    for offset in 0..Coord::N_VALID as u16 {
        let coord = Coord::new(offset).expect("offset in valid range");
        let (i, m, f) = coord.to_axes();
        let packed = ((offset as u32) << 15) | ((i as u32) << 10) | ((m as u32) << 5) | (f as u32);
        writeln!(writer, "{packed:08X}").expect("failed to write line");
    }

    println!("wrote {} lines to {}", Coord::N_VALID, out.display());
}
