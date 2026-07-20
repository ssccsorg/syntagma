use tagma_kv::TagmaKV;

#[test]
fn new_is_empty() {
    let kv = TagmaKV::new();
    assert!(kv.is_empty());
    assert_eq!(kv.len(), 0);
}

#[test]
fn insert_and_get() {
    let mut kv = TagmaKV::new();
    kv.insert("hello", b"world".to_vec());
    assert_eq!(kv.get("hello"), Some(b"world".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn insert_overwrite() {
    let mut kv = TagmaKV::new();
    kv.insert("key", b"v1".to_vec());
    kv.insert("key", b"v2".to_vec());
    assert_eq!(kv.get("key"), Some(b"v2".to_vec()));
    assert_eq!(kv.len(), 1);
}

#[test]
fn remove() {
    let mut kv = TagmaKV::new();
    kv.insert("key", b"value".to_vec());
    assert_eq!(kv.remove("key"), Some(b"value".to_vec()));
    assert!(kv.is_empty());
}

#[test]
fn multiple_keys() {
    let mut kv = TagmaKV::new();
    kv.insert("a", b"1".to_vec());
    kv.insert("b", b"2".to_vec());
    kv.insert("c", b"3".to_vec());
    assert_eq!(kv.len(), 3);
    assert_eq!(kv.get("a"), Some(b"1".to_vec()));
    assert_eq!(kv.get("b"), Some(b"2".to_vec()));
    assert_eq!(kv.get("c"), Some(b"3".to_vec()));
}

#[test]
fn nonexistent_key() {
    let kv = TagmaKV::new();
    assert_eq!(kv.get("nonexistent"), None);
}

#[test]
fn empty_string_returns_none() {
    let mut kv = TagmaKV::new();
    kv.insert("", b"empty".to_vec());
    assert_eq!(kv.get(""), None);
}

#[test]
fn unicode_key() {
    let mut kv = TagmaKV::new();
    kv.insert("\u{d55c}\u{ae00}", b"hangul".to_vec());
    assert_eq!(kv.get("\u{d55c}\u{ae00}"), Some(b"hangul".to_vec()));
}

#[test]
fn short_insert_and_get() {
    let mut kv = TagmaKV::new();
    kv.insert_short("abcd", b"short".to_vec()).unwrap();
    assert_eq!(kv.get_short("abcd"), Some(b"short".to_vec()));
}

#[test]
fn short_key_too_long() {
    let mut kv = TagmaKV::new();
    assert!(kv.insert_short("abcde", b"too long".to_vec()).is_err());
}

#[test]
fn short_get_too_long_returns_none() {
    let kv = TagmaKV::new();
    assert_eq!(kv.get_short("abcde"), None);
}

#[test]
fn clear() {
    let mut kv = TagmaKV::new();
    kv.insert("a", b"1".to_vec());
    kv.insert("b", b"2".to_vec());
    assert_eq!(kv.len(), 2);
    kv.clear();
    assert!(kv.is_empty());
    assert_eq!(kv.get("a"), None);
}

#[test]
fn roundtrip_large_key() {
    let mut kv = TagmaKV::new();
    let key = "this is a relatively long key that exceeds four bytes";
    let val = b"some value".to_vec();
    kv.insert(key, val.clone());
    assert_eq!(kv.get(key), Some(val));
}

#[test]
fn short_and_long_dont_conflict() {
    let mut kv = TagmaKV::new();
    kv.insert_short("abcd", b"short".to_vec()).unwrap();
    assert_eq!(kv.get("abcd"), Some(b"short".to_vec()));
    assert_eq!(kv.get_short("abcd"), Some(b"short".to_vec()));
}

#[test]
fn short_path_modulo_collision_not_visible_to_get() {
    // string_to_short_path uses % 11172, so two different 4-byte keys
    // can collide on the same CoordPath<2>.  get() must NOT see this
    // collision because it only uses the injective byte-wise path.
    let mut kv = TagmaKV::new();
    kv.insert("aaaa", b"value_a".to_vec());
    kv.insert("bbbb", b"value_b".to_vec());
    // get() reads from the injective dyn path only
    assert_eq!(kv.get("aaaa"), Some(b"value_a".to_vec()));
    assert_eq!(kv.get("bbbb"), Some(b"value_b".to_vec()));
    // Both keys are ≤4 bytes, so both also live in short_space.
    // If their CoordPath<2> collides modulo 11172, the later insert
    // overwrites the earlier — but get() is immune because it skips
    // the short path.  We verify by checking the nonexistent key
    // does not leak into get().
    let missing = kv.get("cccc");
    assert_eq!(missing, None, "get must not leak short-path collisions");
}
