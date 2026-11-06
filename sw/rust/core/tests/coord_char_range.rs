// The character a coordinate reads from is the precomposed syllable and nothing
// else.
//
// `from_char` narrows a `char` to a `u16` before the range check, so a
// supplementary-plane character whose low sixteen bits land in the syllable
// block would pass the check. The block is U+AC00..U+D7AF and the truncation is
// invisible to it, so this is the property rather than the arithmetic.

use tagma_core::{Coord, CoordId};

#[test]
fn a_character_above_the_basic_plane_is_not_a_coordinate() {
    // U+1AC00 truncates to 0xAC00, which is the first syllable.
    assert!(
        Coord::from_char('\u{1AC00}').is_none(),
        "a supplementary-plane character was read as the first coordinate"
    );
    // U+1D7AF truncates to 0xD7AF, which is the last syllable.
    assert!(
        Coord::from_char('\u{1D7AF}').is_none(),
        "a supplementary-plane character was read as the last coordinate"
    );
}

#[test]
fn a_text_of_those_characters_is_not_a_canonical_identity() {
    let text: String = '\u{1AC00}'.to_string().repeat(20);
    assert!(
        CoordId::<20>::from_string(&text).is_none(),
        "a twenty-character text outside the block was read as an identity"
    );
}
