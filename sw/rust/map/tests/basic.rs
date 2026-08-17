use tagma_map::coord_gen::CoordKey;
use tagma_map::coord_map_n::CoordMapN;
use tagma_map::{CoordMap, CoordMap2, CoordMapKey, DynCoordMap};

// ── DynCoordMap (dynamic) ─────────────────────────────────────────────────

#[test]
fn dyn_new_is_empty() {
    let map = DynCoordMap::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn dyn_insert_and_get() {
    let mut map = DynCoordMap::new();
    map.insert("hello", b"world".to_vec());
    assert_eq!(map.get("hello"), Some(b"world".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn dyn_insert_overwrite() {
    let mut map = DynCoordMap::new();
    map.insert("key", b"v1".to_vec());
    map.insert("key", b"v2".to_vec());
    assert_eq!(map.get("key"), Some(b"v2".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn dyn_remove() {
    let mut map = DynCoordMap::new();
    map.insert("key", b"value".to_vec());
    assert_eq!(map.remove("key"), Some(b"value".to_vec()));
    assert!(map.is_empty());
}

#[test]
fn dyn_multiple_keys() {
    let mut map = DynCoordMap::new();
    map.insert("a", b"1".to_vec());
    map.insert("b", b"2".to_vec());
    map.insert("c", b"3".to_vec());
    assert_eq!(map.len(), 3);
    assert_eq!(map.get("a"), Some(b"1".to_vec()));
    assert_eq!(map.get("b"), Some(b"2".to_vec()));
    assert_eq!(map.get("c"), Some(b"3".to_vec()));
}

#[test]
fn dyn_nonexistent_key() {
    let map = DynCoordMap::new();
    assert_eq!(map.get("nonexistent"), None);
}

#[test]
fn dyn_empty_string_returns_none() {
    let mut map = DynCoordMap::new();
    map.insert("", b"empty".to_vec());
    assert_eq!(map.get(""), None);
}

#[test]
fn dyn_unicode_key() {
    let mut map = DynCoordMap::new();
    map.insert("\u{d55c}\u{ae00}", b"hangul".to_vec());
    assert_eq!(map.get("\u{d55c}\u{ae00}"), Some(b"hangul".to_vec()));
}

#[test]
fn dyn_clear() {
    let mut map = DynCoordMap::new();
    map.insert("a", b"1".to_vec());
    map.insert("b", b"2".to_vec());
    assert_eq!(map.len(), 2);
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.get("a"), None);
}

#[test]
fn dyn_roundtrip_large_key() {
    let mut map = DynCoordMap::new();
    let key = "this is a relatively long key that exceeds four bytes";
    let val = b"some value".to_vec();
    map.insert(key, val.clone());
    assert_eq!(map.get(key), Some(val));
}

// ── HashMap-compatible insert return ──────────────────────────────────────

#[test]
fn dyn_insert_returns_previous() {
    let mut map = DynCoordMap::new();
    assert_eq!(map.insert("key", b"v1".to_vec()), None);
    assert_eq!(map.insert("key", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn map2_insert_returns_previous() {
    let mut map = CoordMap2::new();
    assert_eq!(map.insert("ky", b"v1".to_vec()), None);
    assert_eq!(map.insert("ky", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn mapn_insert_returns_previous() {
    let mut map = CoordMapN::<3>::new();
    assert_eq!(map.insert("foo", b"v1".to_vec()), None);
    assert_eq!(map.insert("foo", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(map.len(), 1);
}

// ── contains_key ─────────────────────────────────────────────────────────

#[test]
fn dyn_contains_key() {
    let mut map = DynCoordMap::new();
    assert!(!map.contains_key("hello"));
    map.insert("hello", b"world".to_vec());
    assert!(map.contains_key("hello"));
}

#[test]
fn map2_contains_key() {
    let mut map = CoordMap2::new();
    assert!(!map.contains_key("hi"));
    map.insert("hi", b"world".to_vec());
    assert!(map.contains_key("hi"));
}

#[test]
fn map2_contains_key_wrong_length() {
    let map = CoordMap2::new();
    assert!(!map.contains_key("hello"));
}

#[test]
fn mapn_contains_key() {
    let mut map = CoordMapN::<3>::new();
    assert!(!map.contains_key("foo"));
    map.insert("foo", b"bar".to_vec());
    assert!(map.contains_key("foo"));
}

// ── contains_key_by_coordkey ─────────────────────────────────────────────

#[test]
fn map2_contains_key_by_coordkey() {
    let mut map = CoordMap2::new();
    let key = CoordKey::new(*b"hi");
    assert!(!map.contains_key_by_coordkey(&key));
    map.insert_by_coordkey(&key, b"world".to_vec());
    assert!(map.contains_key_by_coordkey(&key));
}

// ── iter ──────────────────────────────────────────────────────────────────

#[test]
fn dyn_iter_empty() {
    let map = DynCoordMap::new();
    assert_eq!(map.iter().count(), 0);
}

#[test]
fn dyn_iter_yields_inserted() {
    let mut map = DynCoordMap::new();
    map.insert("abc", b"123".to_vec());
    map.insert("def", b"456".to_vec());
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| k.clone());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (b"abc".to_vec(), &b"123"[..]));
    assert_eq!(entries[1], (b"def".to_vec(), &b"456"[..]));
}

#[test]
fn mapn_iter_empty() {
    let map: CoordMapN<2> = CoordMapN::new();
    assert_eq!(map.iter().count(), 0);
}

#[test]
fn mapn_iter_yields_inserted() {
    let mut map = CoordMapN::<2>::new();
    map.insert("aa", b"1".to_vec());
    map.insert("bb", b"2".to_vec());
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (*b"aa", &b"1"[..]));
    assert_eq!(entries[1], (*b"bb", &b"2"[..]));
}

// ── CoordMap2 (fixed 2-byte, str API) ─────────────────────────────────────

#[test]
fn map2_new_is_empty() {
    let map = CoordMap2::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn map2_insert_and_get() {
    let mut map = CoordMap2::new();
    map.insert("hi", b"world".to_vec());
    assert_eq!(map.get("hi"), Some(b"world".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn map2_insert_overwrite() {
    let mut map = CoordMap2::new();
    map.insert("ky", b"v1".to_vec());
    map.insert("ky", b"v2".to_vec());
    assert_eq!(map.get("ky"), Some(b"v2".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn map2_remove() {
    let mut map = CoordMap2::new();
    map.insert("ky", b"value".to_vec());
    assert_eq!(map.remove("ky"), Some(b"value".to_vec()));
    assert!(map.is_empty());
}

#[test]
fn map2_multiple_keys() {
    let mut map = CoordMap2::new();
    map.insert("aa", b"1".to_vec());
    map.insert("bb", b"2".to_vec());
    map.insert("cc", b"3".to_vec());
    assert_eq!(map.len(), 3);
    assert_eq!(map.get("aa"), Some(b"1".to_vec()));
    assert_eq!(map.get("bb"), Some(b"2".to_vec()));
    assert_eq!(map.get("cc"), Some(b"3".to_vec()));
}

#[test]
fn map2_nonexistent_key() {
    let map = CoordMap2::new();
    assert_eq!(map.get("no"), None);
}

#[test]
fn map2_wrong_length_returns_none() {
    let map = CoordMap2::new();
    assert_eq!(map.get("hello"), None);
    assert_eq!(map.get("x"), None);
}

#[test]
fn map2_clear() {
    let mut map = CoordMap2::new();
    map.insert("aa", b"1".to_vec());
    map.insert("bb", b"2".to_vec());
    assert_eq!(map.len(), 2);
    map.clear();
    assert!(map.is_empty());
}

// ── CoordMap2: CoordKey API (via CoordMapKey trait) ────────────────────────

#[test]
fn map2_by_coordkey() {
    let mut map = CoordMap2::new();
    let key = CoordKey::new(*b"hi");
    map.insert_by_coordkey(&key, b"world".to_vec());
    assert_eq!(map.get_by_coordkey(&key), Some(b"world".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn map2_by_coordkey_remove() {
    let mut map = CoordMap2::new();
    let key = CoordKey::new(*b"ky");
    map.insert_by_coordkey(&key, b"val".to_vec());
    assert_eq!(map.remove_by_coordkey(&key), Some(b"val".to_vec()));
    assert!(map.is_empty());
}

// ── CoordMapN (fixed N-byte, str API) ─────────────────────────────────────

#[test]
fn mapn_new_is_empty() {
    let map: CoordMapN<3> = CoordMapN::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
}

#[test]
fn mapn_insert_and_get() {
    let mut map = CoordMapN::<3>::new();
    map.insert("foo", b"bar".to_vec());
    assert_eq!(map.get("foo"), Some(b"bar".to_vec()));
    assert_eq!(map.len(), 1);
}

#[test]
fn mapn_wrong_length() {
    let map: CoordMapN<3> = CoordMapN::new();
    assert_eq!(map.get("ab"), None);
    assert_eq!(map.get("abcd"), None);
}

// ── CoordMapN: CoordKey API (via CoordMapKey trait) ────────────────────────

#[test]
fn mapn_by_coordkey() {
    let mut map = CoordMapN::<3>::new();
    let key = CoordKey::new(*b"foo");
    map.insert_by_coordkey(&key, b"bar".to_vec());
    assert_eq!(map.get_by_coordkey(&key), Some(b"bar".to_vec()));
}

// ── Default ──────────────────────────────────────────────────────────────

#[test]
fn dyn_default_is_empty() {
    let map = DynCoordMap::default();
    assert!(map.is_empty());
}

#[test]
fn map2_default_is_empty() {
    let map = CoordMap2::default();
    assert!(map.is_empty());
}

#[test]
fn mapn_default_is_empty() {
    let map: CoordMapN<2> = CoordMapN::default();
    assert!(map.is_empty());
}
