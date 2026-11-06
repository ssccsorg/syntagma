// The canonical text: round trip, the two properties it buys, the exact digits
// it writes, and what it refuses.
//
// The spelling is three base-36 digits an axis, so a text sorts as its
// coordinate does and a prefix of the text is a prefix of the axes. Both depend
// on the alphabet ascending and the width being fixed, which is why the case of
// a digit is not free.

use tagma_core::{Coord, CoordId, CoordPath, DIGITS_PER_AXIS};

fn id_from<const N: usize>(axes: [u16; N]) -> CoordId<N> {
    let mut coords = [Coord::new(0).expect("zero is a coordinate"); N];
    for (slot, value) in coords.iter_mut().zip(axes) {
        *slot = Coord::new(value).expect("a coordinate");
    }
    CoordId(CoordPath::new(coords))
}

fn id_with(first: u16, second: u16) -> CoordId<20> {
    let mut axes = [0u16; 20];
    axes[0] = first;
    axes[1] = second;
    id_from(axes)
}

fn first_window(value: u16) -> String {
    let mut axes = [0u16; 20];
    axes[0] = value;
    id_from(axes).to_string()[..DIGITS_PER_AXIS].to_string()
}

/// A deterministic sample, so the middle of a digit range is visited too.
fn sampled(seed: u64) -> CoordId<20> {
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut axes = [0u16; 20];
    for axis in axes.iter_mut() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *axis = ((state >> 33) % 11_172) as u16;
    }
    id_from(axes)
}

#[test]
fn a_coordinate_round_trips_through_the_text() {
    for (first, second) in [(0, 0), (1, 1), (11_171, 11_171), (8_100, 1_234)] {
        let id = id_with(first, second);
        let text = id.to_string();
        assert_eq!(CoordId::<20>::from_string(&text), Some(id));
    }
}

#[test]
fn the_text_is_three_characters_per_axis() {
    let text = id_with(0, 0).to_string();
    assert_eq!(text.chars().count(), DIGITS_PER_AXIS * 20);
    assert!(text.is_ascii(), "the text holds a byte outside ASCII");
}

#[test]
fn a_prefix_of_the_text_is_a_prefix_of_the_axes() {
    let left = id_with(300, 7).to_string();
    let right = id_with(300, 9).to_string();
    assert_eq!(&left[..3], &right[..3]);
    assert_ne!(&left[3..6], &right[3..6]);
}

#[test]
fn the_text_order_matches_the_coordinate_order() {
    let mut texts: Vec<String> = (0..8).map(|i| id_with(i, 0).to_string()).collect();
    let sorted = texts.clone();
    texts.sort();
    assert_eq!(texts, sorted);
}

#[test]
fn an_uppercase_character_is_refused() {
    let mut text = id_from([11_171u16; 20]).to_string();
    assert_eq!(&text[..3], "8mb");
    text.replace_range(1..2, "M");
    assert_eq!(CoordId::<20>::from_string(&text), None);
}

#[test]
fn a_wrong_length_is_refused() {
    let text = id_with(1, 1).to_string();
    assert_eq!(CoordId::<20>::from_string(&text[..text.len() - 1]), None);
    assert_eq!(CoordId::<20>::from_string(&(text + "0")), None);
    assert_eq!(CoordId::<20>::from_string(""), None);
}

#[test]
fn the_surplus_of_the_alphabet_is_refused() {
    let text = "zzz".repeat(20);
    assert_eq!(CoordId::<20>::from_string(&text), None);
}

#[test]
fn the_digit_carries_render_exactly() {
    assert_eq!(first_window(0), "000");
    assert_eq!(first_window(35), "00z");
    assert_eq!(first_window(36), "010");
    assert_eq!(first_window(1_295), "0zz");
    assert_eq!(first_window(1_296), "100");
    assert_eq!(first_window(11_171), "8mb");
}

#[test]
fn an_axis_position_moves_only_its_own_window() {
    let plain = id_from([0u16; 20]).to_string();
    for axis in 0..20 {
        let mut axes = [0u16; 20];
        axes[axis] = 1;
        let rendered = id_from(axes).to_string();
        let start = axis * DIGITS_PER_AXIS;
        let end = start + DIGITS_PER_AXIS;
        assert_eq!(
            &rendered[..start],
            &plain[..start],
            "axis {axis} moved an earlier window"
        );
        assert_eq!(
            &rendered[start..end],
            "001",
            "axis {axis} did not write its own window"
        );
        assert_eq!(
            &rendered[end..],
            &plain[end..],
            "axis {axis} moved a later window"
        );
    }
}

#[test]
fn every_axis_at_the_maximum_round_trips() {
    let id = id_from([11_171u16; 20]);
    let text = id.to_string();
    assert_eq!(text, "8mb".repeat(20));
    assert_eq!(CoordId::<20>::from_string(&text), Some(id));
}

#[test]
fn a_thousand_sampled_coordinates_round_trip() {
    for seed in 0..1_000 {
        let id = sampled(seed);
        assert_eq!(
            CoordId::<20>::from_string(&id.to_string()),
            Some(id),
            "seed {seed}"
        );
    }
}

#[test]
fn encoding_a_decoded_text_reproduces_the_text() {
    for seed in 0..32 {
        let text = sampled(seed).to_string();
        let id = CoordId::<20>::from_string(&text).expect("a readable text");
        assert_eq!(id.to_string(), text);
    }
}

#[test]
fn a_character_outside_the_alphabet_is_refused_in_every_window() {
    let text = id_with(1, 1).to_string();
    for at in [0, 3, 30, 59] {
        for bad in ["-", " ", "_", "A", "Z", "+"] {
            let mut shaken = text.clone();
            shaken.replace_range(at..at + 1, bad);
            assert_eq!(CoordId::<20>::from_string(&shaken), None, "{bad:?} at {at}");
        }
    }
}

#[test]
fn an_uppercase_copy_of_the_whole_text_is_refused() {
    let text = id_from([11_171u16; 20]).to_string();
    let uppered = text.to_uppercase();
    assert_ne!(uppered, text, "the sample holds no letter");
    assert_eq!(uppered.len(), text.len());
    assert_eq!(CoordId::<20>::from_string(&uppered), None);
}

#[test]
fn a_six_axis_coordinate_round_trips() {
    let id = id_from([1u16, 2, 35, 36, 1_296, 11_171]);
    let text = id.to_string();
    assert_eq!(text, "00100200z0101008mb");
    assert_eq!(CoordId::<6>::from_string(&text), Some(id));
}

#[test]
fn a_depth_the_text_does_not_fill_is_refused() {
    let deep = id_with(1, 1).to_string();
    assert_eq!(CoordId::<6>::from_string(&deep), None);

    let shallow = id_from([1u16, 2, 35, 36, 1_296, 11_171]).to_string();
    assert_eq!(CoordId::<20>::from_string(&shallow), None);
}
