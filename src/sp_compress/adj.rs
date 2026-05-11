use crate::NodeId;

pub const INVALID_ADJ: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
pub struct AdjLink {
    pub edge_idx: u32,
    pub prev: u32,
    pub next: u32,
}

pub struct AdjStore {
    pub head: Vec<u32>,
    pub deg: Vec<u32>,
    pub pool: Vec<AdjLink>,
}

impl AdjStore {
    pub fn new() -> Self {
        AdjStore {
            head: Vec::new(),
            deg: Vec::new(),
            pool: Vec::new(),
        }
    }

    pub fn init(&mut self, n_nodes: u32, edges_capacity: usize) {
        self.head.clear();
        self.head.resize(n_nodes as usize, INVALID_ADJ);
        self.deg.clear();
        self.deg.resize(n_nodes as usize, 0);
        self.pool.clear();

        self.pool.reserve(edges_capacity * 5 / 2 + 16);
    }

    #[inline]
    pub fn insert(&mut self, v: NodeId, edge_idx: u32) -> u32 {
        let idx = self.pool.len() as u32;
        let head_v = self.head[v.idx()];
        self.pool.push(AdjLink {
            edge_idx,
            prev: INVALID_ADJ,
            next: head_v,
        });
        if head_v != INVALID_ADJ {
            self.pool[head_v as usize].prev = idx;
        }
        self.head[v.idx()] = idx;
        self.deg[v.idx()] += 1;
        idx
    }

    #[inline]
    pub fn remove(&mut self, v: NodeId, adj_idx: u32) {
        let (prev, next) = {
            let an = &self.pool[adj_idx as usize];
            (an.prev, an.next)
        };
        if prev != INVALID_ADJ {
            self.pool[prev as usize].next = next;
        } else {
            self.head[v.idx()] = next;
        }
        if next != INVALID_ADJ {
            self.pool[next as usize].prev = prev;
        }

        let an = &mut self.pool[adj_idx as usize];
        an.prev = INVALID_ADJ;
        an.next = INVALID_ADJ;
        self.deg[v.idx()] -= 1;
    }

    #[inline]
    pub fn take_two(&self, v: NodeId) -> (u32, u32) {
        let cur = self.head[v.idx()];
        debug_assert!(cur != INVALID_ADJ);
        let e1 = self.pool[cur as usize].edge_idx;
        let nxt = self.pool[cur as usize].next;
        debug_assert!(nxt != INVALID_ADJ);
        let e2 = self.pool[nxt as usize].edge_idx;
        (e1, e2)
    }

    pub fn drop_storage(&mut self) {
        self.head = Vec::new();
        self.deg = Vec::new();
        self.pool = Vec::new();
    }
}

impl Default for AdjStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    #[test]
    fn insert_remove_basic() {
        let mut s = AdjStore::new();
        s.init(5, 16);
        let i0 = s.insert(NodeId(0), 100);
        let i1 = s.insert(NodeId(0), 101);
        let i2 = s.insert(NodeId(0), 102);
        assert_eq!(s.deg[0], 3);

        let (e1, e2) = s.take_two(NodeId(0));
        assert_eq!(e1, 102);
        assert_eq!(e2, 101);

        s.remove(NodeId(0), i1);
        assert_eq!(s.deg[0], 2);
        let (e1, e2) = s.take_two(NodeId(0));
        assert_eq!(e1, 102);
        assert_eq!(e2, 100);

        s.remove(NodeId(0), i2);
        assert_eq!(s.deg[0], 1);

        s.remove(NodeId(0), i0);
        assert_eq!(s.deg[0], 0);
        assert_eq!(s.head[0], INVALID_ADJ);
    }

    #[test]
    fn deg_independent() {
        let mut s = AdjStore::new();
        s.init(3, 16);
        s.insert(NodeId(0), 100);
        s.insert(NodeId(1), 101);
        s.insert(NodeId(2), 102);
        assert_eq!(s.deg, vec![1, 1, 1]);
    }
}
