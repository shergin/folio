//! Identity: what the revisions of one node have in common.

use std::fmt;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, Weak};

/// A name for a family that is never reused.
///
/// This is the key for anything stored per family: it is `Copy`, hashable and ordered, and
/// it stays valid after the family itself is gone.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FamilyId(NonZeroU64);

/// What all revisions of one node have in common: who it is and where it lives.
///
/// A `Family` is a handle. Cloning it is cheap and gives another handle to the same
/// identity, and two handles are equal when they name the same family. Anything that must
/// outlive a revision keeps a family, never a node: a node is a snapshot. To key a map by
/// family, use its [`id`](Family::id).
///
/// A family is placed under a parent once and never moves. Finding a node therefore costs
/// the depth of the tree, up through families and down through child lists, and the parent
/// of a family is the same in every revision and every candidate. This is the only place in
/// the tree with interior mutability, and only of the write-once kind.
#[derive(Clone)]
pub struct Family(Arc<Identity>);

struct Identity {
    id: FamilyId,
    parent: OnceLock<Weak<Identity>>,
}

impl Family {
    /// A new identity, not yet placed anywhere.
    // Every call makes a different family, which `Default` would not suggest.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Family {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NonZeroU64::new(NEXT.fetch_add(1, Ordering::Relaxed)).expect("family ids exhausted");
        Family(Arc::new(Identity { id: FamilyId(id), parent: OnceLock::new() }))
    }

    /// The family's name.
    pub fn id(&self) -> FamilyId {
        self.0.id
    }

    /// The family of the node that contains nodes of this family, once one has.
    pub fn parent(&self) -> Option<Family> {
        self.0.parent.get()?.upgrade().map(Family)
    }

    /// Records where this family lives.
    ///
    /// # Panics
    ///
    /// If the family already lives under another parent.
    pub(crate) fn place_under(&self, parent: &Family) {
        let placed = self.0.parent.get_or_init(|| Arc::downgrade(&parent.0));
        assert!(std::ptr::eq(placed.as_ptr(), Arc::as_ptr(&parent.0)), "a family never changes its parent");
    }
}

impl PartialEq for Family {
    fn eq(&self, other: &Family) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Family {}

impl fmt::Debug for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Family({})", self.0.id.0)
    }
}
