use crate::coord::Coord;
use alloc::boxed::Box;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// DynCoordSpace — dynamic-depth Coord-addressed space
// ---------------------------------------------------------------------------

/// A collision-free space indexed by a slice of [`Coord`], with dynamic depth.
///
/// Each level is a fixed 11,172-slot array indexed directly by `Coord` —
/// no hashing, no collisions, regardless of depth.
///
/// Unlike [`CoordSpace`] (no_alloc, N=1) and [`CoordSpace6`] (compile-time N=6),
/// the depth is determined at runtime by the length of the path slice.
/// Memory is allocated lazily: only paths that are actually written to
/// consume nodes.
#[derive(Clone, Debug)]
pub struct DynCoordSpace<V> {
    slots: Box<[Option<Slot<V>>]>,
}

#[derive(Clone, Debug)]
enum Slot<V> {
    Leaf(V),
    Node(Box<DynCoordSpace<V>>),
    Both(V, Box<DynCoordSpace<V>>), // holds a value and a child node simultaneously
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl<V> DynCoordSpace<V> {
    /// Creates an empty `DynCoordSpace`.
    #[inline]
    pub fn new() -> Self {
        DynCoordSpace {
            slots: (0..11172)
                .map(|_| None)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}

impl<V> Default for DynCoordSpace<V> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Core: get / insert / remove
// ---------------------------------------------------------------------------

impl<V> DynCoordSpace<V> {
    /// Returns a reference to the value at `path`.
    ///
    /// Returns `None` if `path` is empty.
    ///
    /// Time: O(path.len()) — one array access per coord.
    pub fn at(&self, path: &[Coord]) -> Option<&V> {
        if path.is_empty() {
            return None;
        }
        let mut node = self;
        for (i, &coord) in path.iter().enumerate() {
            let idx = coord.index() as usize;
            match node.slots[idx].as_ref()? {
                Slot::Leaf(v) if i == path.len() - 1 => return Some(v),
                Slot::Both(v, _) if i == path.len() - 1 => return Some(v),
                Slot::Node(child) => node = child,
                Slot::Both(_, child) => node = child,
                _ => return None,
            }
        }
        None
    }

    /// Inserts a value at `path`. Creates intermediate nodes as needed.
    /// Returns the previous value if the exact path already existed.
    ///
    /// # Panics
    ///
    /// Panics if `path` is empty (a path must contain at least one coordinate).
    pub fn place(&mut self, path: &[Coord], value: V) -> Option<V> {
        assert!(
            !path.is_empty(),
            "DynCoordSpace::insert: path must not be empty"
        );
        self.insert_rec(path, 0, value)
    }

    fn insert_rec(&mut self, path: &[Coord], depth: usize, value: V) -> Option<V> {
        let idx = path[depth].index() as usize;
        if depth == path.len() - 1 {
            let slot = &mut self.slots[idx];
            match slot {
                Some(Slot::Leaf(old)) => Some(core::mem::replace(old, value)),
                Some(Slot::Both(old, _)) => Some(core::mem::replace(old, value)),
                Some(Slot::Node(_)) => {
                    *slot = Some(Slot::Leaf(value));
                    None
                }
                None => {
                    *slot = Some(Slot::Leaf(value));
                    None
                }
            }
        } else {
            let slot = &mut self.slots[idx];
            if slot.is_none() {
                *slot = Some(Slot::Node(Box::default()));
            }
            // Take ownership of the slot so we can move values freely.
            let taken = slot.take().unwrap();
            let result;
            *slot = Some(match taken {
                Slot::Node(mut sub) => {
                    result = sub.insert_rec(path, depth + 1, value);
                    Slot::Node(sub)
                }
                Slot::Both(old_val, mut sub) => {
                    result = sub.insert_rec(path, depth + 1, value);
                    Slot::Both(old_val, sub)
                }
                Slot::Leaf(old_val) => {
                    let mut sub: Box<DynCoordSpace<V>> = Box::default();
                    result = sub.insert_rec(path, depth + 1, value);
                    Slot::Both(old_val, sub)
                }
            });
            result
        }
    }

    /// Removes the value at `path`, returning it if present.
    ///
    /// Returns `None` if `path` is empty.
    pub fn vacate(&mut self, path: &[Coord]) -> Option<V> {
        if path.is_empty() {
            return None;
        }
        self.remove_rec(path, 0)
    }

    fn remove_rec(&mut self, path: &[Coord], depth: usize) -> Option<V> {
        let idx = path[depth].index() as usize;
        if depth == path.len() - 1 {
            match self.slots[idx].take() {
                Some(Slot::Leaf(v)) => Some(v),
                Some(Slot::Both(v, child)) => {
                    // Preserve deeper paths by keeping the child as a Node.
                    self.slots[idx] = Some(Slot::Node(child));
                    Some(v)
                }
                _ => None,
            }
        } else {
            match &mut self.slots[idx] {
                Some(Slot::Node(sub)) => sub.remove_rec(path, depth + 1),
                Some(Slot::Both(_, sub)) => sub.remove_rec(path, depth + 1),
                _ => None,
            }
        }
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
    }
}

// ── Iteration (internal) ───────────────────────────────────────────────

impl<V> DynCoordSpace<V> {
    /// Returns the number of entries across all depths.
    /// O(entries) — walks the tree counting occupied leaf slots.
    pub fn entry_count(&self) -> usize {
        self.count_rec()
    }

    fn count_rec(&self) -> usize {
        let mut count = 0;
        for slot in self.slots.iter() {
            match slot {
                Some(Slot::Leaf(_)) => count += 1,
                Some(Slot::Node(sub)) => count += sub.count_rec(),
                Some(Slot::Both(_, sub)) => count += 1 + sub.count_rec(),
                None => {}
            }
        }
        count
    }

    /// Returns an iterator over all `(path, value)` pairs.
    /// Paths are yielded in depth-first, coordinate-ascending order.
    pub fn iter(&self) -> DynIter<'_, V> {
        let mut entries = Vec::new();
        let mut path = Vec::new();
        self.collect_iter(&mut path, &mut entries);
        DynIter {
            entries: entries.into_iter(),
        }
    }

    fn collect_iter<'a>(&'a self, path: &mut Vec<Coord>, out: &mut Vec<(Vec<Coord>, &'a V)>) {
        for (i, slot) in self.slots.iter().enumerate() {
            let coord = Coord::new(i as u16).unwrap();
            match slot {
                Some(Slot::Leaf(v)) => {
                    path.push(coord);
                    out.push((path.clone(), v));
                    path.pop();
                }
                Some(Slot::Node(sub)) => {
                    path.push(coord);
                    sub.collect_iter(path, out);
                    path.pop();
                }
                Some(Slot::Both(v, sub)) => {
                    path.push(coord);
                    out.push((path.clone(), v));
                    sub.collect_iter(path, out);
                    path.pop();
                }
                None => {}
            }
        }
    }
}

/// An iterator over `(path, value)` pairs in a `DynCoordSpace`.
pub struct DynIter<'a, V> {
    entries: alloc::vec::IntoIter<(Vec<Coord>, &'a V)>,
}

impl<'a, V> Iterator for DynIter<'a, V> {
    type Item = (Vec<Coord>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}
