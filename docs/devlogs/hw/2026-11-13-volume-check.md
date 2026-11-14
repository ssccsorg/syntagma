# The volume's addresses, checked in fabric

The checksum image checks the decoder against the reference anchors, which is a
statement about the valid range of `tagma_core`. This image checks the values a
store actually wrote, so what the die confirms is that the addresses on a
medium are coordinates.

## What runs

The host reads the keys of a volume and takes the coordinate values out of
them. A key is `facts/f_<sixty base-36 digits>.fact`, and the sixty digits are
twenty coordinates of three digits each, so the values come off the medium
rather than out of the engine. Those values become the ROM of
`tagma_volume_verify_top`, which decodes each one and accumulates
`(i << 10) | (m << 5) | f`.

The host computes the same sum by division. The die computes it with the
reciprocal multiplies of the pipelined decoder, so agreement is between two
arithmetic paths and not a design compared with itself.

A value at or above 11,172 is outside the alphabet. It is counted and kept out
of the sum, so one corrupted value changes the sum and raises the count, and
either one clears `pin_match`.

## The measurement

A volume of 11,172 records under the declared layout, the first 512 of them
read back off the medium, which is 10,240 coordinate values.

| | |
|---|---|
| Values in the ROM | 10,240 |
| DP16KD | 9 of 56 (16%) |
| TRELLIS_FF / COMB | 219 / 448 |
| Timing at a 40 MHz constraint | 76.99 MHz |
| Host checksum | `00002BFC` |
| `pin_match` | 1 |
| `pin_invalid` | 0 |

## The negative control

One value at index 5000 was changed to `2ba4`, which is 11,172 and the first
value outside the alphabet, and the image was rebuilt at the same expected sum.

| | `pin_match` | `pin_invalid` |
|---|---|---|
| The volume's values | 1 | 0 |
| One value at 11,172 | 0 | 1 |

So the verdict responds to the data, and a pass is a comparison that ran.

## The defect this found

The first build reported `pin_match` zero with `pin_invalid` zero, which is the
signature of a sum that is wrong while every value still reads as valid. The
decoder takes the Unicode code point of a coordinate, not its value, so
presenting 0..11,171 directly made the offset subtraction wrap:

    0x0000 - 0xAC00 = 0x5400, and the decoder keeps the low fourteen bits

so the decode ran on `value + 0x1400` for every value, and 5,120 of the 11,172
coordinates decoded to a different coordinate rather than to an invalid one.
Adding `0xAC00` once, before the decoder, is the fix. The sum then reproduces
the anchors' constant over the whole valid range, which is the same `065BEA26`
the sweep image reports.

## What it does not prove

No rate. The image reads one value per clock and reports a verdict; it is not
compared with anything on a host, and the hardware series already records that
the software wins both latency and sweep throughput.

The ROM is built by the flow from the host's values, so the claim is about the
fit between the medium and the circuit, not about a decoder reading a live
medium.

512 of the 11,172 records are checked, because a ROM of all 223,440 values does
not fit the part. The count is stated rather than implied.
