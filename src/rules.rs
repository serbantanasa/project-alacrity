use crate::hypergraph::Hypergraph;
use rand::{rngs::StdRng, Rng, seq::SliceRandom};

#[derive(Debug)]
pub struct Rule {
    pub after: Vec<Vec<usize>>,
    pub remove: bool,
    pub node: usize,
}

pub fn generate_rule(hg: &Hypergraph, pattern: &[Vec<usize>], rng: &mut StdRng) -> Rule {
    let active = hg.active_nodes();
    let r = rng.gen::<f64>(); // Use float for precise probability
    if r < 0.5 { // 50% Split
        let edge = &pattern[0];
        let new_node = hg.get_max_node_id() + 1;
        let mut after = Vec::new();
        if edge.len() >= 2 {
            after.push(vec![edge[0], new_node]); // Connect to first node
            after.push(vec![new_node, *active.choose(rng).unwrap_or(&edge[1])]); // Connect to random active
        }
        Rule { after, remove: true, node: 0 }
    } else if r < 0.9 { // 40% Toggle Add
        let node = *active.choose(rng).unwrap_or(&pattern[0][0]);
        let mut after = Vec::new();
        after.push(vec![node, node]);
        Rule { after, remove: false, node }
    } else { // 10% Toggle Remove
        let node = *active.choose(rng).unwrap_or(&pattern[0][0]);
        let has_self = hg.edges_slice().iter().any(|e| e.len() == 2 && e[0] == node && e[1] == node);
        let degree = hg.degrees()[node];
        let mut after = Vec::new();
        if has_self && degree > 1 { // Only remove if connected
            Rule { after, remove: true, node }
        } else {
            after.push(vec![node, node]); // Fallback to add if not safe to remove
            Rule { after, remove: false, node }
        }
    }
}