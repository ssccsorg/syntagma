# tagma-kv: the bridge

Tagma KV is the first practical bridge between the standard key-value paradigm and
Tagma's coordinate space. It answers a single make-or-break question.

## The premise

Tagma KV cannot use hash functions. It converts string keys to Coord sequences via
byte-wise or fixed-size mapping (ByteWise, CoordKey<N>). The concern has always been:

> If string-to-Coord conversion is slower than string-to-hash, Tagma KV can
> never replace HashMap in the general case. The bridge collapses.

This is a hard constraint: the conversion must be at least as fast as SipHash-2-4
(the standard Rust HashMap hasher), and ideally faster. If not, every Tagma KV
operation carries a baseline penalty that has nothing to do with storage or indexing.

## The benchmark (ARMv8.4-A Firestorm)

```
Benchmark: single get, 1000 iterations per sample
```

| Path | Cost | Breakdown |
|------|------|-----------|
| `HashMap<String>` get | 23.8 ns | SipHash + bucket lookup |
| `CoordKV2` get via `"str"` | 22.5 ns | str-to-CoordKey conversion + slot load |
| Raw CoordSpace2 slot load | 1.07 ns | pure array access (no conversion) |

The str-to-CoordKey conversion accounts for ~21 of the 22.5 ns. The remaining 1.07 ns
is the actual slot access the pure Tagma promise.

The result: string-to-Coord is slightly faster than string-to-hash. This is
intuitive in hindsight: mapping 2 bytes to a Coord via from(&str) is a bounds check
and a copy; SipHash-2-4 must process every byte through 4 rounds of compression and
2 rounds of finalization. For 2-byte keys, the former is strictly less work.

For longer keys (CoordKV dynamic, ByteWise strategy), the conversion incurs one
Coord per byte, totaling O(len) allocation and copy. HashMap SipHash also
processes every byte, but without allocation. The dynamic path is therefore
predictably slower by the allocation cost.

```
                    get latency (lower is better)

  CoordKV2             ████████████████████  22.5 ns
  HashMap<String>      █████████████████████  23.8 ns
  DynCoordKV (4B)      ████████████████████████████████████████  43.2 ns
  DynCoordKV (14B)     ██████████████████████████████████████████████████████  72.1 ns
  Raw CoordSpace2      █   1.07 ns
```

## Why the conversion cost matches hash

```
str-to-hash (SipHash):
  for each byte:
    compress with XOR, rotate, word mixing
  finalize with 2 more rounds
  to 1 hash value

str-to-CoordKey (ByteWise):
  for each byte:
    push byte as u16 into Vec<Coord>
  to N Coord values (N = key length)
```

SipHash does more work per byte but produces one compact value. ByteWise does less
work per byte but produces N values. For 2-byte keys (CoordKV2), the former is more
expensive. For 4+ byte keys, allocation cost tips the balance toward HashMap.

The key insight: at the 2-byte boundary which covers common short identifiers,
status codes, language pairs, two-letter country codes, 65,536 possible values
CoordKV2 beats HashMap on both speed and collision guarantees.

## The bridge

This result is strategically important not because Tagma KV replaces HashMap in every
workload, but because it proves the bridge exists at all. Before this measurement,
there was no evidence that a hashless string-to-address mapping could be competitive.
The gap could have been 10x or 100x. It is 0.94x (slightly faster).

With the bridge established:

| Before (no bridge) | After (bridge exists) |
|---|---|
| Tagma is a specialized system for numeric Coord addresses | Tagma KV accepts &str at HashMap-competitive speed |
| Existing KV workloads must be redesigned | Existing KV workloads migrate transparently |
| Tagma spatial indexing requires dedicated data pipeline | Spatial indexing is a property of the same KV store, zero extra cost |

## What the bridge enables

Every entry stored via Tagma KV is stored in Tagma coordinate space. This means
the same store supports:

- **Prefix scan**: iter_prefix("us") returns all entries under that prefix in O(matched)
  time. HashMap must scan all entries.
- **Axis projection**: query by decomposition of the stored Coord values into their
  three structural axes (initial, medial, final). These axes can represent independent
  application dimensions.
- **Range query**: consecutive Coord values in linearized space correspond to contiguous
  address ranges, enabling sequential read patterns that hash-based stores cannot provide.
- **Zero-collision guarantee**: the mapping is structurally injective for the fixed-key
  variants (CoordKV2, CoordKVN). No hash collision handling, no load factor, no
  rehashing.

These capabilities are not add-ons. They are the same coordinate space, accessed
through the same API. An application that starts with kv.insert("hi", value) today
can later query kv.iter_prefix("h") without changing a single byte of stored data.

## Summary

The premise is satisfied. The str-to-Coord conversion is competitive with str-to-hash,
with the fixed-2-byte variant slightly faster than SipHash. Tagma KV is not a compromise
for compatibility. It is the entry point through which existing KV workloads can adopt
Tagma coordinate space and, over time, exploit its spatial indexing capabilities
without migration cost.

```
tagma-kv: string key to Coord sequence to Tagma coordinate space to spatial indexing
           ^                                    ^
           HashMap-competitive                   zero extra cost
```
