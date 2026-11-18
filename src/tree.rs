//! The tree: its revisions, the commit, and the edit of one view by identity.

use std::sync::{Arc, Mutex};

use crate::family::Family;
use crate::shadow_node_kind::ShadowNodeKind;
use crate::props::ViewProps;
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};
use crate::view::ViewShadowNode;

/// One committed state of a tree: a root and its place in the order of commits.
#[derive(Clone, Debug)]
pub struct Revision {
    /// The root of the tree in this revision.
    pub root: Arc<ShadowNodeKind>,
    /// How many commits led to this revision.
    pub number: u64,
}

/// The tree moved on while a candidate was being prepared. Holds the revision to rebase
/// the candidate onto.
#[derive(Debug)]
pub struct Stale(pub Revision);

/// The one mutable cell: which revision is current.
///
/// The root of a tree is a plain view that stands outside every window. It is never mounted
/// and exists to hold the windows; everything visible is inside some window.
#[derive(Debug)]
pub struct ShadowTree {
    current: Mutex<Revision>,
}

impl ShadowTree {
    /// A tree with nothing in it.
    pub fn new() -> ShadowTree {
        let root = ViewShadowNode::new(ViewProps::default(), []);
        ShadowTree { current: Mutex::new(Revision { root: Arc::new(root.into()), number: 0 }) }
    }

    /// The current revision. It can be read from any thread, for as long as it is held.
    pub fn current(&self) -> Revision {
        self.current.lock().unwrap().clone()
    }

    /// Makes `root` current if nothing was committed since `base`.
    ///
    /// The candidate is built and laid out before this call, on whichever thread holds it;
    /// the lock covers the swap only.
    ///
    /// # Errors
    ///
    /// [`Stale`] if another commit came first.
    pub fn try_commit(&self, base: &Revision, root: Arc<ShadowNodeKind>) -> Result<Revision, Stale> {
        let mut current = self.current.lock().unwrap();
        if current.number != base.number {
            return Err(Stale(current.clone()));
        }
        *current = Revision { root, number: base.number + 1 };
        Ok(current.clone())
    }
}

impl Default for ShadowTree {
    fn default() -> Self {
        ShadowTree::new()
    }
}

/// Edits the view of `target` in the tree under `root`, which becomes the next revision.
///
/// Views on the path from the root to the target are copied if they are shared. Everything
/// else stays pointer-equal to the previous revision, and the layout engine learns from the
/// path which levels changed.
///
/// Returns `false` if `target` is not a view of this tree. A span is edited through its
/// text.
pub fn edit(root: &mut Arc<ShadowNodeKind>, target: &Family, edit: impl FnOnce(&mut ShadowNodeKind)) -> bool {
    // Upwards, by identity alone: the families between the target and the root.
    let mut path = Vec::new();
    let (mut id, mut parent) = (target.id(), target.parent());
    while id != root.family().id() {
        path.push(id);
        let Some(family) = parent else { return false };
        (id, parent) = (family.id(), family.parent());
    }

    // Downwards, reading: the child index at each level.
    let mut indices = Vec::with_capacity(path.len());
    let mut node: &ShadowNodeKind = root;
    for id in path.iter().rev() {
        let children = node.layout_parts().children;
        let Some(index) = children.slots().iter().position(|slot| slot.node.family().id() == *id) else {
            return false;
        };
        indices.push(index);
        node = &children.slots()[index].node;
    }

    // Downwards, writing: copy-on-write along the path.
    let mut node = Arc::make_mut(root);
    for index in indices {
        node = node.layout_parts_mut().children.make_mut()[index].node_mut();
    }
    edit(node);
    true
}
