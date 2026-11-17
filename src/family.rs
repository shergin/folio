//! Identity: what the revisions of one node have in common.

use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, Weak};

/// A name for a family that is never reused.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FamilyId(NonZeroU64);

/// What all revisions of one node have in common: who it is and where it lives.
///
/// Anything that must outlive a revision refers to the family, never to a node: a node is a
/// snapshot.
///
/// A family is placed under a parent once and never moves. Finding a node therefore costs
/// the depth of the tree, up through families and down through child lists, and the parent
/// of a family is the same in every revision and every candidate. This is the only place in
/// the tree with interior mutability, and only of the write-once kind.
#[derive(Debug)]
pub struct Family {
    id: FamilyId,
    parent: OnceLock<Weak<Family>>,
}

impl Family {
    /// A new identity, not yet placed anywhere.
    pub fn new() -> Arc<Family> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NonZeroU64::new(NEXT.fetch_add(1, Ordering::Relaxed)).expect("family ids exhausted");
        Arc::new(Family { id: FamilyId(id), parent: OnceLock::new() })
    }

    /// The family's name.
    pub fn id(&self) -> FamilyId {
        self.id
    }

    /// The family of the node that contains nodes of this family, once one has.
    pub fn parent(&self) -> Option<Arc<Family>> {
        self.parent.get()?.upgrade()
    }

    /// Records where this family lives.
    ///
    /// # Panics
    ///
    /// If the family already lives under another parent.
    pub(crate) fn place_under(&self, parent: &Arc<Family>) {
        let placed = self.parent.get_or_init(|| Arc::downgrade(parent));
        assert!(std::ptr::eq(placed.as_ptr(), Arc::as_ptr(parent)), "a family never changes its parent");
    }
}
