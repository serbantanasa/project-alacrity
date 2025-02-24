mod hypergraph;
mod rules;

use hypergraph::Hypergraph;
use rand::{rngs::StdRng, SeedableRng};
use rules::generate_rule;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut hg = Hypergraph::new();

    // Start with one doubly self-connected node
    let initial_node = hg.add_node(0);
    hg.add_hyperedge(vec![initial_node, initial_node]);
    hg.add_hyperedge(vec![initial_node, initial_node]);
    println!(
        "Initial: Nodes: {}, Edges: {}, Max Degree: {} (Node {}), Active Nodes: {:?}, Degrees: {:?}, Sample: {:?}",
        hg.node_count(),
        hg.edge_count(),
        hg.max_degree(),
        hg.max_degree_node(),
        hg.active_nodes(),
        hg.degrees(),
        hg.edges_slice().get(..std::cmp::min(5, hg.edge_count())).unwrap_or(&[])
    );

    for step in 1..=30 {
        let active = hg.active_nodes();
        if active.is_empty() {
            println!("Step {}: No active nodes!", step);
            break;
        }
        let n = active[0];
        let local = hg.local_connected(n, 1, hg.max_degree(), &mut rng);
        if local.is_empty() {
            continue;
        }
        let pattern_indices: Vec<usize> = local.into_iter().take(1).collect();
        let pattern: Vec<Vec<usize>> = pattern_indices
            .iter()
            .filter_map(|&i| hg.get_edge(i).cloned())
            .collect();
        
        let rule = generate_rule(&hg, &pattern, &mut rng);
        if rule.remove {
            hg.remove_hyperedges(&pattern_indices);
        }
        for edge in &rule.after {
            hg.add_hyperedge(edge.clone());
        }
        hg.cleanup(step);

        println!(
            "Step {}: Nodes: {}, Edges: {}, Max Degree: {} (Node {}), Rule: {}, Active Nodes: {:?}, Degrees: {:?}, Sample: {:?}",
            step,
            hg.node_count(),
            hg.edge_count(),
            hg.max_degree(),
            hg.max_degree_node(),
            if rule.after.len() > 0 && rule.after[0].len() >= 2 && rule.after[0][0] == rule.after[0][1] { "Toggle Add" } else if rule.remove && !rule.after.is_empty() { "Split" } else { "Toggle Remove" },
            hg.active_nodes(),
            hg.degrees(),
            hg.edges_slice().get(..std::cmp::min(5, hg.edge_count())).unwrap_or(&[])
        );
    }

    plot_final(&hg)?;
    Ok(())
}

fn plot_final(hg: &Hypergraph) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("final.png", (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let active = hg.active_nodes();
    let n = active.len() as f32;
    let radius = n as f32 * 2.0;
    let positions: Vec<(f32, f32)> = active.iter().enumerate().map(|(i, &_)| {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / n;
        (radius * angle.cos(), radius * angle.sin())
    }).collect();

    let mut chart = ChartBuilder::on(&root)
        .caption("Final Hypergraph", ("sans-serif", 20).into_font())
        .build_cartesian_2d(-radius..radius, -radius..radius)?;

    // Draw edges first (directed with tiny arrows)
    for edge in hg.edges_slice() {
        if edge.len() >= 2 {
            let from_idx = active.iter().position(|&x| x == edge[0]).unwrap_or(0);
            let to_idx = active.iter().position(|&x| x == edge[1]).unwrap_or(0);
            let from_pos = positions[from_idx];
            let to_pos = positions[to_idx];
            if from_idx != to_idx {
                chart.draw_series(std::iter::once(
                    PathElement::new(vec![from_pos, to_pos], BLACK.stroke_width(1))
                ))?;
                let dx = to_pos.0 - from_pos.0;
                let dy = to_pos.1 - from_pos.1;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    let nx = dx / len;
                    let ny = dy / len;
                    let arrow_size = 0.5;
                    let p1 = (to_pos.0 - arrow_size * nx + arrow_size * ny / 2.0, to_pos.1 - arrow_size * ny - arrow_size * nx / 2.0);
                    let p2 = (to_pos.0 - arrow_size * nx - arrow_size * ny / 2.0, to_pos.1 - arrow_size * ny + arrow_size * nx / 2.0);
                    chart.draw_series(std::iter::once(
                        Polygon::new(vec![to_pos, p1, p2], BLACK.filled())
                    ))?;
                }
            }
        }
    }

    // Count self-relationships per node
    let mut self_counts = vec![0; hg.get_max_node_id() + 1];
    for edge in hg.edges_slice() {
        if edge.len() >= 2 && edge[0] == edge[1] {
            self_counts[edge[0]] += 1;
        }
    }

    // Draw nodes and self-loops (self-loops on top)
    for (i, &node) in active.iter().enumerate() {
        let pos = positions[i];
        // Node circle (gray outline for degree 0)
        let degree = hg.degrees()[node];
        if degree == 0 {
            chart.draw_series(std::iter::once(
                Circle::new(pos, 6, BLUE.filled()) // Filled blue dot
            ))?;
            chart.draw_series(std::iter::once(
                Circle::new(pos, 7, GREEN.stroke_width(1)) // Gray outline
            ))?;
        } else {
            chart.draw_series(std::iter::once(
                Circle::new(pos, 5, BLUE.filled()) // Standard blue dot
            ))?;
        }
        // Self-relationship circles (smaller, red for visibility)
        for r in 0..self_counts[node] {
            let radius = 8 + r as u32 * 4;
            chart.draw_series(std::iter::once(
                Circle::new(pos, radius as i32, RED.stroke_width(1))
            ))?;
        }
    }

    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE)
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}