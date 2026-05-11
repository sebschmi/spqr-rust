use crate::sp_compress::types::{CompressionInput, CompressionStats, SpTree};
use crate::{build_spqr_raw, Graph, NodeId, SpqrResult};

pub struct CompressAndSpqrResult {
    pub macro_tree: SpTree,

    pub core_spqr: Option<SpqrResult>,

    pub core_node_remap: Vec<u32>,
    pub core_node_inv: Vec<NodeId>,
}

impl CompressAndSpqrResult {
    pub fn stats(&self) -> &CompressionStats {
        &self.macro_tree.stats
    }
}

pub fn compress_and_build_spqr(input: &CompressionInput) -> CompressAndSpqrResult {
    compress_and_build_spqr_borrowed(input.n_nodes, &input.edges, &input.contractible)
}

pub fn compress_and_build_spqr_borrowed(
    n_nodes: u32,
    input_edges: &[crate::sp_compress::types::InputEdge],
    contractible: &[u8],
) -> CompressAndSpqrResult {
    let cr = crate::sp_compress::reduction::compress_borrowed(n_nodes, input_edges, contractible);
    let macro_tree = cr.tree;
    let (core_spqr, core_node_remap, core_node_inv) = build_core_spqr_parts(n_nodes, &macro_tree);

    CompressAndSpqrResult {
        macro_tree,
        core_spqr,
        core_node_remap,
        core_node_inv,
    }
}

#[inline]
pub(crate) fn build_core_spqr_parts(
    n_nodes: u32,
    macro_tree: &SpTree,
) -> (Option<SpqrResult>, Vec<u32>, Vec<NodeId>) {
    if macro_tree.stats.fully_sp_reducible != 0 || macro_tree.core_edges.is_empty() {
        return (None, Vec::new(), Vec::new());
    }

    let n_orig = n_nodes as usize;
    let mut remap = vec![u32::MAX; n_orig];
    let mut inv: Vec<NodeId> = Vec::with_capacity(macro_tree.core_nodes.len());
    for v in &macro_tree.core_nodes {
        remap[v.idx()] = inv.len() as u32;
        inv.push(*v);
    }

    let n_core = inv.len();
    let m_core = macro_tree.core_edges.len();

    let mut graph = Graph::with_capacity(n_core, m_core);
    graph.add_nodes_fast(n_core);
    for ce in &macro_tree.core_edges {
        let u_remap = remap[ce.u as usize];
        let v_remap = remap[ce.v as usize];
        debug_assert!(u_remap != u32::MAX);
        debug_assert!(v_remap != u32::MAX);
        graph.add_edge(NodeId(u_remap), NodeId(v_remap));
    }

    let spqr = build_spqr_raw(&graph);

    (Some(spqr), remap, inv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sp_compress::types::InputEdge;
    use crate::EdgeId;

    fn mk_input(n_nodes: u32, edges: &[(u32, u32)], contractible_set: &[u32]) -> CompressionInput {
        let mut input = CompressionInput {
            n_nodes,
            edges: Vec::with_capacity(edges.len()),
            contractible: vec![0u8; n_nodes as usize],
        };
        for &v in contractible_set {
            input.contractible[v as usize] = 1;
        }
        for (i, &(u, v)) in edges.iter().enumerate() {
            input.edges.push(InputEdge {
                u: NodeId(u),
                v: NodeId(v),
                original_edge_id: EdgeId(i as u32),
            });
        }
        input
    }

    #[test]
    fn theta_is_fully_reducible_so_no_spqr() {
        let input = mk_input(5, &[(0, 1), (1, 2), (2, 3), (0, 4), (4, 3)], &[1, 2, 4]);
        let r = compress_and_build_spqr(&input);
        assert_eq!(r.macro_tree.stats.fully_sp_reducible, 1);
        assert!(r.core_spqr.is_none());
    }

    #[test]
    fn k4_is_r_so_spqr_built() {
        let input = mk_input(
            4,
            &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            &[0, 1, 2, 3],
        );
        let r = compress_and_build_spqr(&input);
        assert_eq!(r.macro_tree.stats.fully_sp_reducible, 0);
        assert!(r.core_spqr.is_some());

        let spqr = r.core_spqr.as_ref().unwrap();

        assert!(!spqr.tree.is_empty());
    }

    #[test]
    fn chain_is_fully_reducible() {
        let n = 100u32;
        let mut edges = Vec::new();
        for i in 0..=n {
            edges.push((i, i + 1));
        }
        let mut contr = Vec::new();
        for i in 1..=n {
            contr.push(i);
        }
        let input = mk_input(n + 2, &edges, &contr);
        let r = compress_and_build_spqr(&input);
        assert_eq!(r.macro_tree.stats.fully_sp_reducible, 1);
        assert!(r.core_spqr.is_none());
    }
}
