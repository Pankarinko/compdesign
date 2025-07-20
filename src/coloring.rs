use crate::{ir::IRFunction, liveness::analyze_func};

#[derive(Debug)]
struct Node {
    ind: usize,
    neighbors: Vec<usize>,
}

pub fn color_func(f: &mut IRFunction) -> Vec<usize> {
    let live_temps = analyze_func(&mut f.instructions);
    let mut edges = build_interference(&live_temps, f.num_temps);
    let nodes = order_nodes(&mut edges);
    color_greedy(&nodes)
}

/*Creates a vector with a color for every node in increasing node order */
fn color_greedy(ordered_nodes: &[Node]) -> Vec<usize> {
    let mut coloring = Vec::new();
    coloring.resize_with(ordered_nodes.len(), || 0);
    for n in ordered_nodes.iter() {
        let mut min_color = 0;
        while n
            .neighbors
            .iter()
            .any(|neighbor| coloring[*neighbor] == min_color)
        {
            min_color += 1;
        }
        coloring[n.ind] = min_color;
    }
    coloring
}

/* Creates a vector that decsribes the node order */
fn order_nodes(edges: &mut Vec<Vec<usize>>) -> Vec<Node> {
    let num_nodes = edges.len();
    let mut weights: Vec<(usize, usize)> = Vec::new();
    let mut nodes_ordered = Vec::new();
    let mut x = 0;
    weights.resize_with(num_nodes, || {
        x += 1;
        ((x - 1), 0)
    });
    for _ in 0..num_nodes {
        let (i, (n, _)) = weights.iter().enumerate().max_by_key(|x| *x).unwrap();
        let current = edges.swap_remove(i);
        nodes_ordered.push(Node {
            ind: *n,
            neighbors: current,
        });
        weights.swap_remove(i);
        for node in nodes_ordered.last().unwrap().neighbors.iter() {
            if let Some(w) = weights.iter_mut().find(|x| x.0 == *node) {
                w.1 += 1;
            }
        }
    }
    nodes_ordered
}

fn build_interference(live_temps: &[Vec<usize>], num_temps: usize) -> Vec<Vec<usize>> {
    let mut edges: Vec<Vec<usize>> = Vec::new();
    edges.resize_with(num_temps, Vec::new);
    for line in live_temps.iter() {
        line.iter().for_each(|t| {
            line.iter().for_each(|neighbor| {
                if *neighbor != *t && !edges[*t].contains(neighbor) {
                    edges[*t].push(*neighbor);
                }
            })
        });
    }
    edges
}
