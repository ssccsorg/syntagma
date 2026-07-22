use tagma_kv::coord_gen::CoordKey;
use tagma_kv::coord_kv_n::CoordKVN;
use tagma_kv::{CoordKV, CoordKV2, CoordKVKey, DynCoordKV};

// ── DynCoordKV (dynamic) ─────────────────────────────────────────────────

#[test]
fn dyn_new_is_empty() {
    let kv = DynCoordKV::new();
    assert!(kv.is_empty());
    assert_eq!(kv.len(), 0);
}

#[test]
fn dyn_insert_and_get() {
    let mut kv = DynCoordKV::new();
    kv.insert("hello", b"world".to_vec());
    assert_eq!(kv.get("hello"), Some(b"world".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn dyn_insert_overwrite() {
    let mut kv = DynCoordKV::new();
    kv.insert("key", b"v1".to_vec());
    kv.insert("key", b"v2".to_vec());
    assert_eq!(kv.get("key"), Some(b"v2".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn dyn_remove() {
    let mut kv = DynCoordKV::new();
    kv.insert("key", b"value".to_vec());
    assert_eq!(kv.remove("key"), Some(b"value".to_vec()));
    assert!(kv.is_empty());
}

#[test]
fn dyn_multiple_keys() {
    let mut kv = DynCoordKV::new();
    kv.insert("a", b"1".to_vec());
    kv.insert("b", b"2".to_vec());
    kv.insert("c", b"3".to_vec());
    assert_eq!(kv.len(), 3);
    assert_eq!(kv.get("a"), Some(b"1".to_vec()));
    assert_eq!(kv.get("b"), Some(b"2".to_vec()));
    assert_eq!(kv.get("c"), Some(b"3".to_vec()));
}

#[test]
fn dyn_nonexistent_key() {
    let kv = DynCoordKV::new();
    assert_eq!(kv.get("nonexistent"), None);
}

#[test]
fn dyn_empty_string_returns_none() {
    let mut kv = DynCoordKV::new();
    kv.insert("", b"empty".to_vec());
    assert_eq!(kv.get(""), None);
}

#[test]
fn dyn_unicode_key() {
    let mut kv = DynCoordKV::new();
    kv.insert("\u{d55c}\u{ae00}", b"hangul".to_vec());
    assert_eq!(kv.get("\u{d55c}\u{ae00}"), Some(b"hangul".to_vec()));
}

#[test]
fn dyn_clear() {
    let mut kv = DynCoordKV::new();
    kv.insert("a", b"1".to_vec());
    kv.insert("b", b"2".to_vec());
    assert_eq!(kv.len(), 2);
    kv.clear();
    assert!(kv.is_empty());
    assert_eq!(kv.get("a"), None);
}

#[test]
fn dyn_roundtrip_large_key() {
    let mut kv = DynCoordKV::new();
    let key = "this is a relatively long key that exceeds four bytes";
    let val = b"some value".to_vec();
    kv.insert(key, val.clone());
    assert_eq!(kv.get(key), Some(val));
}

// ── HashMap-compatible insert return ──────────────────────────────────────

#[test]
fn dyn_insert_returns_previous() {
    let mut kv = DynCoordKV::new();
    assert_eq!(kv.insert("key", b"v1".to_vec()), None);
    assert_eq!(kv.insert("key", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kv2_insert_returns_previous() {
    let mut kv = CoordKV2::new();
    assert_eq!(kv.insert("ky", b"v1".to_vec()), None);
    assert_eq!(kv.insert("ky", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kvn_insert_returns_previous() {
    let mut kv = CoordKVN::<3>::new();
    assert_eq!(kv.insert("foo", b"v1".to_vec()), None);
    assert_eq!(kv.insert("foo", b"v2".to_vec()), Some(b"v1".to_vec()));
    assert_eq!(kv.len(), 1);
}

// ── contains_key ─────────────────────────────────────────────────────────

#[test]
fn dyn_contains_key() {
    let mut kv = DynCoordKV::new();
    assert!(!kv.contains_key("hello"));
    kv.insert("hello", b"world".to_vec());
    assert!(kv.contains_key("hello"));
}

#[test]
fn kv2_contains_key() {
    let mut kv = CoordKV2::new();
    assert!(!kv.contains_key("hi"));
    kv.insert("hi", b"world".to_vec());
    assert!(kv.contains_key("hi"));
}

#[test]
fn kv2_contains_key_wrong_length() {
    let kv = CoordKV2::new();
    assert!(!kv.contains_key("hello"));
}

#[test]
fn kvn_contains_key() {
    let mut kv = CoordKVN::<3>::new();
    assert!(!kv.contains_key("foo"));
    kv.insert("foo", b"bar".to_vec());
    assert!(kv.contains_key("foo"));
}

// ── contains_key_by_coordkey ─────────────────────────────────────────────

#[test]
fn kv2_contains_key_by_coordkey() {
    let mut kv = CoordKV2::new();
    let key = CoordKey::new(*b"hi");
    assert!(!kv.contains_key_by_coordkey(&key));
    kv.insert_by_coordkey(&key, b"world".to_vec());
    assert!(kv.contains_key_by_coordkey(&key));
}

// ── iter ──────────────────────────────────────────────────────────────────

#[test]
fn dyn_iter_empty() {
    let kv = DynCoordKV::new();
    assert_eq!(kv.iter().count(), 0);
}

#[test]
fn dyn_iter_yields_inserted() {
    let mut kv = DynCoordKV::new();
    kv.insert("abc", b"123".to_vec());
    kv.insert("def", b"456".to_vec());
    let mut entries: Vec<_> = kv.iter().collect();
    entries.sort_by_key(|(k, _)| k.clone());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (b"abc".to_vec(), &b"123"[..]));
    assert_eq!(entries[1], (b"def".to_vec(), &b"456"[..]));
}

#[test]
fn kvn_iter_empty() {
    let kv: CoordKVN<2> = CoordKVN::new();
    assert_eq!(kv.iter().count(), 0);
}

#[test]
fn kvn_iter_yields_inserted() {
    let mut kv = CoordKVN::<2>::new();
    kv.insert("aa", b"1".to_vec());
    kv.insert("bb", b"2".to_vec());
    let mut entries: Vec<_> = kv.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (*b"aa", &b"1"[..]));
    assert_eq!(entries[1], (*b"bb", &b"2"[..]));
}

// ── CoordKV2 (fixed 2-byte, str API) ─────────────────────────────────────

#[test]
fn kv2_new_is_empty() {
    let kv = CoordKV2::new();
    assert!(kv.is_empty());
    assert_eq!(kv.len(), 0);
}

#[test]
fn kv2_insert_and_get() {
    let mut kv = CoordKV2::new();
    kv.insert("hi", b"world".to_vec());
    assert_eq!(kv.get("hi"), Some(b"world".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kv2_insert_overwrite() {
    let mut kv = CoordKV2::new();
    kv.insert("ky", b"v1".to_vec());
    kv.insert("ky", b"v2".to_vec());
    assert_eq!(kv.get("ky"), Some(b"v2".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kv2_remove() {
    let mut kv = CoordKV2::new();
    kv.insert("ky", b"value".to_vec());
    assert_eq!(kv.remove("ky"), Some(b"value".to_vec()));
    assert!(kv.is_empty());
}

#[test]
fn kv2_multiple_keys() {
    let mut kv = CoordKV2::new();
    kv.insert("aa", b"1".to_vec());
    kv.insert("bb", b"2".to_vec());
    kv.insert("cc", b"3".to_vec());
    assert_eq!(kv.len(), 3);
    assert_eq!(kv.get("aa"), Some(b"1".to_vec()));
    assert_eq!(kv.get("bb"), Some(b"2".to_vec()));
    assert_eq!(kv.get("cc"), Some(b"3".to_vec()));
}

#[test]
fn kv2_nonexistent_key() {
    let kv = CoordKV2::new();
    assert_eq!(kv.get("no"), None);
}

#[test]
fn kv2_wrong_length_returns_none() {
    let kv = CoordKV2::new();
    assert_eq!(kv.get("hello"), None);
    assert_eq!(kv.get("x"), None);
}

#[test]
fn kv2_clear() {
    let mut kv = CoordKV2::new();
    kv.insert("aa", b"1".to_vec());
    kv.insert("bb", b"2".to_vec());
    assert_eq!(kv.len(), 2);
    kv.clear();
    assert!(kv.is_empty());
}

// ── CoordKV2: CoordKey API (via CoordKVKey trait) ────────────────────────

#[test]
fn kv2_by_coordkey() {
    let mut kv = CoordKV2::new();
    let key = CoordKey::new(*b"hi");
    kv.insert_by_coordkey(&key, b"world".to_vec());
    assert_eq!(kv.get_by_coordkey(&key), Some(b"world".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kv2_by_coordkey_remove() {
    let mut kv = CoordKV2::new();
    let key = CoordKey::new(*b"ky");
    kv.insert_by_coordkey(&key, b"val".to_vec());
    assert_eq!(kv.remove_by_coordkey(&key), Some(b"val".to_vec()));
    assert!(kv.is_empty());
}

// ── CoordKVN (fixed N-byte, str API) ─────────────────────────────────────

#[test]
fn kvn_new_is_empty() {
    let kv: CoordKVN<3> = CoordKVN::new();
    assert!(kv.is_empty());
    assert_eq!(kv.len(), 0);
}

#[test]
fn kvn_insert_and_get() {
    let mut kv = CoordKVN::<3>::new();
    kv.insert("foo", b"bar".to_vec());
    assert_eq!(kv.get("foo"), Some(b"bar".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn kvn_wrong_length() {
    let kv: CoordKVN<3> = CoordKVN::new();
    assert_eq!(kv.get("ab"), None);
    assert_eq!(kv.get("abcd"), None);
}

// ── CoordKVN: CoordKey API (via CoordKVKey trait) ────────────────────────

#[test]
fn kvn_by_coordkey() {
    let mut kv = CoordKVN::<3>::new();
    let key = CoordKey::new(*b"foo");
    kv.insert_by_coordkey(&key, b"bar".to_vec());
    assert_eq!(kv.get_by_coordkey(&key), Some(b"bar".to_vec()));
}

// ── Default ──────────────────────────────────────────────────────────────

#[test]
fn dyn_default_is_empty() {
    let kv = DynCoordKV::default();
    assert!(kv.is_empty());
}

#[test]
fn kv2_default_is_empty() {
    let kv = CoordKV2::default();
    assert!(kv.is_empty());
}

#[test]
fn kvn_default_is_empty() {
    let kv: CoordKVN<2> = CoordKVN::default();
    assert!(kv.is_empty());
}
