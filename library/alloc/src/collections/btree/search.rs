use core::borrow::Borrow;

use super::node::{marker, ForceResult::*, Handle, NodeRef};

use SearchResult::*;

pub enum SearchResult<BorrowType, K, V, FoundType, GoDownType> {
    Found(Handle<NodeRef<BorrowType, K, V, FoundType>, marker::KV>),
    GoDown(Handle<NodeRef<BorrowType, K, V, GoDownType>, marker::Edge>),
}

/// Looks up a given key in a (sub)tree headed by the given node, recursively.
/// Returns a `Found` with the handle of the matching KV, if any. Otherwise,
/// returns a `GoDown` with the handle of the possible leaf edge where the key
/// belongs.
///
/// The result is meaningful only if the tree is ordered by key, like the tree
/// in a `BTreeMap` is.
pub fn search_tree<BorrowType, K, V, Q: ?Sized>(
    mut node: NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
    key: &Q,
) -> SearchResult<BorrowType, K, V, marker::LeafOrInternal, marker::Leaf>
where
    Q: Ord,
    K: Borrow<Q>,
{
    loop {
        match search_node(node, key) {
            Found(handle) => return Found(handle),
            GoDown(handle) => match handle.force() {
                Leaf(leaf) => return GoDown(leaf),
                Internal(internal) => {
                    node = internal.descend();
                    continue;
                }
            },
        }
    }
}

/// Looks up a given key in a given node, without recursion.
/// Returns a `Found` with the handle of the matching KV, if any. Otherwise,
/// returns a `GoDown` with the handle of the edge where the key might be found
/// (if the node is internal) or where the key can be inserted.
///
/// The result is meaningful only if the tree is ordered by key, like the tree
/// in a `BTreeMap` is.
pub fn search_node<BorrowType, K, V, Type, Q: ?Sized>(
    node: NodeRef<BorrowType, K, V, Type>,
    key: &Q,
) -> SearchResult<BorrowType, K, V, Type, Type>
where
    Q: Ord,
    K: Borrow<Q>,
{
    match search_binary(&node, key) {
        (idx, true) => Found(unsafe { Handle::new_kv(node, idx) }),
        (idx, false) => GoDown(unsafe { Handle::new_edge(node, idx) }),
    }
}

/// Returns either the KV index in the node at which the key (or an equivalent)
/// exists and `true`, or the edge index where the key belongs and `false`.
///
/// The result is meaningful only if the tree is ordered by key, like the tree
/// in a `BTreeMap` is.
fn search_binary<BorrowType, K, V, Type, Q: ?Sized>(
    node: &NodeRef<BorrowType, K, V, Type>,
    key: &Q,
) -> (usize, bool)
where
    Q: Ord,
    K: Borrow<Q>,
{
    // This function is defined over all borrow types (immutable, mutable, owned).
    match unsafe { node.reborrow().keys_as_slice() }.binary_search_by_key(&key, Borrow::borrow) {
        Ok(i) => (i, true),
        Err(i) => (i, false),
    }
}
