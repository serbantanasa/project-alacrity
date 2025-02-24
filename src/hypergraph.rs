use std::collections::HashSet;
use rand::Rng;

#[derive(Debug)]
pub struct Hypergraph {
    edges: Vec<Vec<usize>>,
    degrees: Vec<usize>,
    max_node_id: usize,
    recent_nodes: Vec<(usize, usize)>, // (node, step_added)
}

impl Hypergraph {
    pub fn new() -> Self {
        Hypergraph {
            edges: Vec::new(),
            degrees: Vec::new(),
            max_node_id: 0,
            recent_nodes: Vec::new(),
        }
    }

    pub fn add_hyperedge(&mut self, edge: Vec<usize>) {
        if edge.len() < 2 { return; }
        for &node in &edge {
            if node >= self.degrees.len() {
                self.degrees.resize(node + 1, 0);
                self.max_node_id = self.max_node_id.max(node);
            }
            self.degrees[node] = self.degrees[node].saturating_add(1);
        }
        self.edges.push(edge);
    }

    pub fn remove_hyperedges(&mut self, indices: &[usize]) {
        let mut removed = HashSet::new();
        for &i in indices.iter().rev() {
            if i < self.edges.len() && !removed.contains(&i) {
                for &node in &self.edges[i] {
                    if self.degrees[node] > 0 {
                        self.degrees[node] = self.degrees[node].saturating_sub(1);
                    }
                }
                self.edges.swap_remove(i);
                removed.insert(i);
            }
        }
    }

    pub fn add_node(&mut self, step: usize) -> usize {
        self.max_node_id += 1;
        self.degrees.push(0);
        self.recent_nodes.push((self.max_node_id, step));
        println!("Added node {} at step {}", self.max_node_id, step);
        self.max_node_id
    }

    pub fn node_count(&self) -> usize {
        self.degrees.iter().filter(|&&d| d > 0).count()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn local_connected(&self, node: usize, max_size: usize, _max_degree: usize, _rng: &mut impl Rng) -> Vec<usize> {
        if node >= self.degrees.len() || self.degrees[node] == 0 {
            return Vec::new();
        }
        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.contains(&node) && result.len() < max_size {
                result.push(i);
                visited.insert(i);
                for &n in edge {
                    queue.push(n);
                }
            }
        }
        while !queue.is_empty() && result.len() < max_size {
            let n = queue.remove(0);
            for (i, edge) in self.edges.iter().enumerate() {
                if !visited.contains(&i) && edge.contains(&n) {
                    result.push(i);
                    visited.insert(i);
                    for &m in edge {
                        if !queue.contains(&m) {
                            queue.push(m);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn cleanup(&mut self, current_step: usize) {
        let mut new_edges = Vec::new();
        let mut node_map = vec![None; self.max_node_id + 1];
        let mut new_id = 0;
        for i in 0..self.degrees.len() {
            let is_recent = self.recent_nodes.iter().any(|&(n, step)| n == i && current_step - step <= 10);
            if self.degrees[i] > 0 || is_recent {
                node_map[i] = Some(new_id);
                new_id += 1;
            }
        }
        for edge in &self.edges {
            let mut new_edge = Vec::new();
            for &node in edge {
                if let Some(mapped) = node_map[node] {
                    new_edge.push(mapped);
                } else {
                    println!("Node {} pruned at step {}", node, current_step);
                }
            }
            if new_edge.len() >= 2 {
                new_edges.push(new_edge);
            }
        }
        self.edges = new_edges;
        self.degrees = vec![0; new_id];
        for edge in &self.edges {
            for &node in edge {
                self.degrees[node] += 1;
            }
        }
        self.max_node_id = new_id.saturating_sub(1);
        self.recent_nodes.retain(|&(_, step)| current_step - step <= 10);
    }

    pub fn get_edge(&self, index: usize) -> Option<&Vec<usize>> {
        self.edges.get(index)
    }

    pub fn edges_slice(&self) -> &[Vec<usize>] {
        &self.edges
    }

    pub fn max_degree(&self) -> usize {
        self.degrees.iter().copied().max().unwrap_or(0)
    }

    pub fn max_degree_node(&self) -> usize {
        self.degrees.iter().enumerate().max_by_key(|&(_, d)| d).map(|(i, _)| i).unwrap_or(0)
    }

    pub fn active_nodes(&self) -> Vec<usize> {
        self.degrees.iter().enumerate().filter(|&(_, &d)| d > 0).map(|(i, _)| i).collect()
    }

    pub fn get_max_node_id(&self) -> usize {
        self.max_node_id
    }

    pub fn degrees(&self) -> &Vec<usize> {
        &self.degrees
    }
}