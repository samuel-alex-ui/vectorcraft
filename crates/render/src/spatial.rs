//! Conservative, painter-order-preserving index for large world-map groups.
//!
//! This index contains only transient render bounds. It never owns vector geometry and
//! must not be serialized. Cached indices live beside a node's Arc-identity geometry
//! cache entry, so changing a group invalidates the index automatically.

use std::mem::size_of;

use crate::Rect;

/// Small leaves avoid excessive tree nodes while keeping viewport queries cheap.
const LEAF_SIZE: usize = 12;
/// Spatial indices share a bounded auxiliary-memory cache in the renderer.
/// This does not include the existing document nodes or geometry cache.
pub(crate) const CACHE_BUDGET: usize = 64 << 20;
/// Keep individual index construction bounded; exceptionally large
/// containers retain the renderer's existing conservative culling path.
pub(crate) const MAX_INDEXED_MEMBERS: usize = 262_144;

struct Item {
    index: usize,
    bounds: Rect,
}

pub(crate) struct SpatialIndex {
    root: Option<Box<Branch>>,
    /// Objects without usable finite bounds are always visited, never silently lost.
    always: Vec<usize>,
    total: usize,
    /// Count tree allocations once on construction, not on each pan frame.
    bytes: usize,
}

struct Branch {
    bounds: Rect,
    left: Option<Box<Branch>>,
    right: Option<Box<Branch>>,
    items: Vec<usize>,
}

fn valid(r: Rect) -> bool {
    r.x0.is_finite() && r.y0.is_finite() && r.x1.is_finite() && r.y1.is_finite() && r.x0 <= r.x1 && r.y0 <= r.y1
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

fn union(a: Rect, b: Rect) -> Rect {
    Rect::new(a.x0.min(b.x0), a.y0.min(b.y0), a.x1.max(b.x1), a.y1.max(b.y1))
}

impl Branch {
    fn build(items: &mut [Item]) -> Self {
        let mut bounds = items.first().map_or(Rect::new(0.0, 0.0, 0.0, 0.0), |item| item.bounds);
        for item in items.iter().skip(1) {
            bounds = union(bounds, item.bounds);
        }
        if items.len() <= LEAF_SIZE {
            return Self { bounds, left: None, right: None, items: items.iter().map(|x| x.index).collect() };
        }
        let horizontal = bounds.x1 - bounds.x0 >= bounds.y1 - bounds.y0;
        items.sort_unstable_by(|a, b| if horizontal { a.bounds.x0.total_cmp(&b.bounds.x0) } else { a.bounds.y0.total_cmp(&b.bounds.y0) });
        let middle = items.len() / 2;
        let (left, right) = items.split_at_mut(middle);
        Self { bounds, left: Some(Box::new(Self::build(left))), right: Some(Box::new(Self::build(right))), items: Vec::new() }
    }

    fn heap_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.items.capacity().saturating_mul(size_of::<usize>()))
            .saturating_add(self.left.as_ref().map_or(0, |node| node.heap_bytes()))
            .saturating_add(self.right.as_ref().map_or(0, |node| node.heap_bytes()))
    }

    fn query(&self, viewport: Rect, result: &mut Vec<usize>) {
        if !overlap(self.bounds, viewport) {
            return;
        }
        if let Some(left) = &self.left {
            left.query(viewport, result);
        }
        if let Some(right) = &self.right {
            right.query(viewport, result);
        }
        // Leaves are conservative: the renderer performs its original bounds test.
        result.extend_from_slice(&self.items);
    }
}

impl SpatialIndex {
    pub(crate) fn new(bounds: impl IntoIterator<Item = (usize, Option<Rect>)>, total: usize) -> Self {
        let mut items = Vec::new();
        let mut always = Vec::new();
        for (index, rect) in bounds {
            match rect {
                Some(rect) if valid(rect) => items.push(Item { index, bounds: rect }),
                _ => always.push(index),
            }
        }
        let root = (!items.is_empty()).then(|| Box::new(Branch::build(&mut items)));
        let bytes = size_of::<Self>()
            .saturating_add(always.capacity().saturating_mul(size_of::<usize>()))
            .saturating_add(root.as_ref().map_or(0, |branch| branch.heap_bytes()));
        Self { root, always, total, bytes }
    }

    /// Estimate the live memory owned by this index (excluding document geometry).
    pub(crate) fn memory_bytes(&self) -> usize {
        self.bytes
    }

    /// Return indices in their *original* order, preserving painter's order and blending.
    /// Non-finite viewport coordinates conservatively visit every child.
    pub(crate) fn candidates(&self, viewport: Rect) -> Vec<usize> {
        if !valid(viewport) {
            return (0..self.total).collect();
        }
        let mut out = self.always.clone();
        if let Some(root) = &self.root {
            root.query(viewport, &mut out);
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_memory_tracks_tree_and_fallback_storage() {
        let index = SpatialIndex::new((0..256).map(|i| (i, Some(Rect::new(i as f64, 0.0, i as f64 + 1.0, 10.0)))), 256);
        assert!(index.memory_bytes() >= size_of::<SpatialIndex>() + size_of::<Branch>());
        let unknown = SpatialIndex::new((0..256).map(|i| (i, None)), 256);
        assert!(unknown.memory_bytes() >= size_of::<SpatialIndex>() + 256 * size_of::<usize>());
        assert!(index.memory_bytes() <= CACHE_BUDGET);
    }

    #[test]
    fn candidates_keep_painter_order_and_edge_contacts() {
        let bounds = (0..256).map(|i| {
            let x = (255 - i) as f64 * 10.0;
            (i, Some(Rect::new(x, 0.0, x + 5.0, 5.0)))
        });
        let index = SpatialIndex::new(bounds, 256);
        let ids = index.candidates(Rect::new(100.0, 0.0, 130.0, 5.0));
        assert!(!ids.is_empty());
        assert!(ids.windows(2).all(|p| p[0] < p[1]));
        for i in 0..256 {
            let x = (255 - i) as f64 * 10.0;
            if x <= 130.0 && x + 5.0 >= 100.0 {
                assert!(ids.contains(&i), "lost intersecting node {i}");
            }
        }
        assert!(ids.len() < 40, "index did not prune distant nodes");
    }

    #[test]
    fn unknown_bounds_and_bad_viewports_are_conservative() {
        let bounds = [(0, None), (1, Some(Rect::new(0.0, 0.0, 1.0, 1.0))), (2, Some(Rect::new(f64::NAN, 0.0, 2.0, 2.0)))];
        let index = SpatialIndex::new(bounds, 3);
        assert_eq!(index.candidates(Rect::new(500.0, 500.0, 510.0, 510.0)), vec![0, 2]);
        assert_eq!(index.candidates(Rect::new(f64::NAN, 0.0, 5.0, 5.0)), vec![0, 1, 2]);
    }

    #[test]
    fn every_intersecting_rectangle_is_included() {
        let bounds: Vec<_> = (0..1000)
            .map(|i| {
                let x = ((i * 37) % 431) as f64;
                let y = ((i * 59) % 337) as f64;
                (i, Some(Rect::new(x, y, x + (i % 41 + 1) as f64, y + (i % 23 + 1) as f64)))
            })
            .collect();
        let index = SpatialIndex::new(bounds.iter().copied(), bounds.len());
        for x in [0.0, 50.0, 200.0, 400.0] {
            for y in [0.0, 30.0, 200.0, 300.0] {
                let view = Rect::new(x, y, x + 20.0, y + 15.0);
                let candidates = index.candidates(view);
                for (id, rect) in &bounds {
                    if rect.is_some_and(|r| overlap(r, view)) {
                        assert!(candidates.binary_search(id).is_ok(), "missed node {id} at {x}, {y}");
                    }
                }
            }
        }
    }
}
