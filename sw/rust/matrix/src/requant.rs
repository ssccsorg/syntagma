//! The rescale between an accumulator and an element.
//!
//! A product accumulates in `i32` and the next layer consumes `i8`. The step
//! between them is one multiply by a fixed point value followed by a right shift,
//! and it is where a quantized model's arithmetic is easiest to get subtly wrong:
//! the product does not fit in thirty-two bits, and a rounding that is correct for
//! a magnitude is wrong for a signed value.
//!
//! The contract is stated as one result, so any implementation that reproduces it is
//! conformant whatever it does inside. The result is the accumulator times the
//! multiplier, plus a rounding constant of half the shifted width, moved right by
//! the shift, and saturated into `i8`. The intermediate is sixty-four bits wide; a
//! thirty-two bit target reaches it through a high multiply and a low multiply
//! rather than through a wider register, which is why the shift bound below is a
//! bound on the intermediate rather than on a register.
//!
//! The TFLite line and its gemmlowp ancestor write this as
//! `(accumulator * multiplier + rounding) >> shift` with `rounding = 1 << (shift - 1)`
//! over a signed accumulator, which is the form this module implements. Their other
//! spelling, a doubling high multiply composed with a power-of-two divide, rounds
//! the same product another way; a model carrying that spelling is converted to this
//! one before it is run here.

/// The fixed point rescale between an accumulator and an element.
///
/// `multiplier` and `shift` are the two halves of one fixed point value: the rescale
/// multiplies by `multiplier` and then divides by two to the `shift`. A value below
/// one is carried as a small multiplier with a large shift, and a value above one as
/// a large multiplier with a small shift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requant {
    multiplier: i32,
    shift: u32,
}

impl Requant {
    /// The largest shift whose rounding constant still fits beside the product.
    ///
    /// The product of two thirty-two bit values occupies at most sixty-two bits, so
    /// a rounding constant derived from a shift above sixty-two pushes the sum past
    /// the sixty-four bit intermediate.
    pub const MAX_SHIFT: u32 = 62;

    /// A rescale that multiplies by `multiplier` and moves right by `shift` bits.
    ///
    /// # Panics
    ///
    /// Panics if `shift` is above [`Self::MAX_SHIFT`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tagma_matrix::Requant;
    ///
    /// // A value a little below one, carried as a quantized model carries it.
    /// let requant = Requant::new(38_997_123, 27);
    /// assert_eq!(requant.requantize(300), 87);
    /// assert_eq!(requant.requantize(-300), -87);
    /// ```
    pub const fn new(multiplier: i32, shift: u32) -> Self {
        assert!(
            shift <= Self::MAX_SHIFT,
            "tagma-matrix: shift exceeds the sixty-four bit intermediate"
        );
        Self { multiplier, shift }
    }

    /// The value the accumulator is multiplied by.
    pub const fn multiplier(&self) -> i32 {
        self.multiplier
    }

    /// The bits the product is moved right.
    pub const fn shift(&self) -> u32 {
        self.shift
    }

    /// The rescale of one accumulator, saturated into an element.
    ///
    /// The rounding constant is added before the shift, and the shift is arithmetic
    /// and therefore floors, so the product is divided by two to the shift and
    /// rounded half up: a product exactly half way between two integers moves toward
    /// positive infinity. That is the convention the TFLite line states, and it is
    /// not symmetric about zero, since half of `-1` is `0` rather than `-1`. A shift
    /// of zero adds no rounding constant and takes the product as it stands.
    #[inline]
    pub const fn requantize(&self, accumulator: i32) -> i8 {
        let product = (accumulator as i64) * (self.multiplier as i64);
        let rounding = if self.shift == 0 {
            0
        } else {
            1i64 << (self.shift - 1)
        };
        saturate((product + rounding) >> self.shift)
    }
}

/// The element nearest a wide value, saturating at the ends of the element range.
#[inline]
const fn saturate(value: i64) -> i8 {
    if value > i8::MAX as i64 {
        i8::MAX
    } else if value < i8::MIN as i64 {
        i8::MIN
    } else {
        value as i8
    }
}

/// The rescale of a run of accumulators, saturated into elements.
///
/// # Panics
///
/// Panics if `out` is shorter than `accumulators`. A destination that cannot hold
/// the run is refused rather than filled part way, so a layer that under-sizes its
/// output fails where it happens rather than leaving the tail at a stale value.
#[inline]
pub fn requantize_into(accumulators: &[i32], requant: &Requant, out: &mut [i8]) {
    assert!(
        out.len() >= accumulators.len(),
        "tagma-matrix: the destination is shorter than the accumulators"
    );
    for (accumulator, element) in accumulators.iter().zip(out.iter_mut()) {
        *element = requant.requantize(*accumulator);
    }
}
