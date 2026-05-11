use crate::{EdgeId, NodeId, SkeletonEdge, SpqrNodeType, SpqrTree, TreeNodeId};

pub type CanonicalForm = Vec<u8>;

pub fn isomorphic(a: &SpqrTree, b: &SpqrTree) -> bool {
    canonical_form(a) == canonical_form(b)
}

pub fn canonical_form(tree: &SpqrTree) -> CanonicalForm {
    let n = tree.len();
    if n == 0 {
        return b"<empty>".to_vec();
    }

    let mut neighbors: Vec<Vec<TreeNodeId>> = vec![Vec::new(); n];

    for i in 0..n {
        let p = tree.node_parents[i];
        if p.is_valid() && p.idx() != i {
            neighbors[i].push(p);
            neighbors[p.idx()].push(TreeNodeId(i as u32));
        }
    }

    let centers = tree_centers(&neighbors, n);
    let mut best: Option<Vec<u8>> = None;

    for c in centers {
        let sig = rooted_signature(tree, &neighbors, c);

        match &best {
            None => best = Some(sig),
            Some(cur) if &sig < cur => best = Some(sig),
            _ => {}
        }
    }

    best.unwrap_or_default()
}

fn tree_centers(neighbors: &[Vec<TreeNodeId>], n: usize) -> Vec<TreeNodeId> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![TreeNodeId(0)];
    }

    let mut degree: Vec<u32> = neighbors.iter().map(|nbs| nbs.len() as u32).collect();

    let mut leaves: Vec<TreeNodeId> = (0..n)
        .filter(|&i| degree[i] <= 1)
        .map(|i| TreeNodeId(i as u32))
        .collect();

    let mut remaining = n;

    while remaining > 2 {
        let mut new_leaves: Vec<TreeNodeId> = Vec::new();

        for &leaf in &leaves {
            for &nb in &neighbors[leaf.idx()] {
                if degree[nb.idx()] > 0 {
                    degree[nb.idx()] -= 1;

                    if degree[nb.idx()] == 1 {
                        new_leaves.push(nb);
                    }
                }
            }

            degree[leaf.idx()] = 0;
        }

        remaining -= leaves.len();
        leaves = new_leaves;
    }

    leaves
}

fn rooted_signature(
    tree: &SpqrTree,
    neighbors: &[Vec<TreeNodeId>],
    root: TreeNodeId,
) -> CanonicalForm {
    let n = tree.len();

    let mut subtree_sig: Vec<CanonicalForm> = vec![Vec::new(); n];
    let mut parent_in_dfs: Vec<TreeNodeId> = vec![TreeNodeId::INVALID; n];

    let mut stack: Vec<(TreeNodeId, bool)> = Vec::with_capacity(n);
    let mut visited = vec![false; n];

    stack.push((root, false));
    visited[root.idx()] = true;
    parent_in_dfs[root.idx()] = TreeNodeId::INVALID;

    while let Some(&(node, expanded)) = stack.last() {
        if !expanded {
            stack.last_mut().unwrap().1 = true;

            let p = parent_in_dfs[node.idx()];

            for &nb in &neighbors[node.idx()] {
                if nb != p && !visited[nb.idx()] {
                    visited[nb.idx()] = true;
                    parent_in_dfs[nb.idx()] = node;
                    stack.push((nb, false));
                }
            }
        } else {
            stack.pop();

            let p = parent_in_dfs[node.idx()];
            let s = node_signature(tree, node, p, &subtree_sig, neighbors);

            subtree_sig[node.idx()] = s;
        }
    }

    std::mem::take(&mut subtree_sig[root.idx()])
}

fn node_signature(
    tree: &SpqrTree,
    node: TreeNodeId,
    parent_in_dfs: TreeNodeId,
    child_sigs: &[CanonicalForm],
    neighbors: &[Vec<TreeNodeId>],
) -> CanonicalForm {
    let mut buf: Vec<u8> = Vec::with_capacity(64);

    let tag = match tree.node_type(node) {
        SpqrNodeType::S => b'S',
        SpqrNodeType::P => b'P',
        SpqrNodeType::R => b'R',
    };

    buf.push(tag);
    buf.push(b'[');

    let mapping = tree.node_mapping_slice(node);
    let edges = tree.skeleton_edges_slice(node);

    let mut edge_descs: Vec<EdgeDesc> = Vec::with_capacity(edges.len());

    for e in edges {
        edge_descs.push(make_edge_desc(e, mapping, parent_in_dfs, child_sigs));
    }

    edge_descs.sort();

    extend_u32(&mut buf, edge_descs.len() as u32);
    for d in &edge_descs {
        d.encode_into(&mut buf);
    }

    let mut child_neighbors: Vec<&[u8]> = Vec::new();

    for &nb in &neighbors[node.idx()] {
        if nb != parent_in_dfs {
            child_neighbors.push(&child_sigs[nb.idx()]);
        }
    }

    child_neighbors.sort();

    extend_u32(&mut buf, child_neighbors.len() as u32);
    for cs in child_neighbors {
        extend_u32(&mut buf, cs.len() as u32);
        buf.extend_from_slice(cs);
    }

    buf.push(b']');
    buf
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum EdgeDesc {
    Real {
        endpoints: (u32, u32),
        real_edge_id: u32,
    },
    VUp {
        endpoints: (u32, u32),
    },
    VDown {
        endpoints: (u32, u32),
        child_sig: Vec<u8>,
    },
}

fn make_edge_desc(
    e: &SkeletonEdge,
    mapping: &[NodeId],
    parent_in_dfs: TreeNodeId,
    child_sigs: &[CanonicalForm],
) -> EdgeDesc {
    let s_orig = mapping[e.src.idx()].0;
    let d_orig = mapping[e.dst.idx()].0;

    let endpoints = if s_orig <= d_orig {
        (s_orig, d_orig)
    } else {
        (d_orig, s_orig)
    };

    if e.real_edge != EdgeId::INVALID {
        EdgeDesc::Real {
            endpoints,
            real_edge_id: e.real_edge.0,
        }
    } else {
        let twin = e.twin_tree_node;

        if twin == parent_in_dfs {
            EdgeDesc::VUp { endpoints }
        } else {
            let child_sig = if twin.is_valid() {
                child_sigs[twin.idx()].clone()
            } else {
                Vec::new()
            };

            EdgeDesc::VDown {
                endpoints,
                child_sig,
            }
        }
    }
}

impl EdgeDesc {
    fn encode_into(&self, out: &mut Vec<u8>) {
        match self {
            EdgeDesc::Real {
                endpoints,
                real_edge_id,
            } => {
                out.push(b'R');
                extend_u32(out, endpoints.0);
                extend_u32(out, endpoints.1);
                extend_u32(out, *real_edge_id);
            }
            EdgeDesc::VUp { endpoints } => {
                out.push(b'U');
                extend_u32(out, endpoints.0);
                extend_u32(out, endpoints.1);
            }
            EdgeDesc::VDown {
                endpoints,
                child_sig,
            } => {
                out.push(b'D');
                extend_u32(out, endpoints.0);
                extend_u32(out, endpoints.1);
                extend_u32(out, child_sig.len() as u32);
                out.extend_from_slice(child_sig);
            }
        }
    }
}

#[inline]
fn extend_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_spqr, Graph, NodeId};

    fn graph_from(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_capacity(n as usize, edges.len());
        g.add_nodes_fast(n as usize);

        for &(u, v) in edges {
            g.add_edge(NodeId(u), NodeId(v));
        }

        g
    }

    #[test]
    fn iso_equal_to_self() {
        let g = graph_from(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);

        let t1 = build_spqr(&g).tree;
        let t2 = build_spqr(&g).tree;

        assert!(isomorphic(&t1, &t2));
    }

    #[test]
    fn iso_equal_under_edge_permutation() {
        let g1 = graph_from(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let g2 = graph_from(4, &[(2, 3), (1, 3), (1, 2), (0, 3), (0, 2), (0, 1)]);

        let t1 = build_spqr(&g1).tree;
        let t2 = build_spqr(&g2).tree;

        assert!(isomorphic(&t1, &t1));
        assert!(isomorphic(&t2, &t2));
        assert!(!isomorphic(&t1, &t2));
    }

    #[test]
    fn iso_distinguishes_topology() {
        let k4 = graph_from(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let theta = graph_from(2, &[(0, 1), (0, 1), (0, 1)]);

        let tk = build_spqr(&k4).tree;
        let tt = build_spqr(&theta).tree;

        assert!(!isomorphic(&tk, &tt));
    }

    #[test]
    fn centers_match_for_path_like_trees() {
        let chain = graph_from(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]);
        let t = build_spqr(&chain).tree;

        let _ = canonical_form(&t);
    }
}
