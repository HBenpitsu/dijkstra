use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};

use crate::graph::*;
use crate::mutable_heap::*;

#[derive(Debug, Clone)]
pub struct DijkstraNode {
    distance: Box<usize>,
    heap_id: usize,
}
impl Display for DijkstraNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.distance)
    }
}

#[derive(Debug)]
pub struct DijkstraArc {
    weight: usize,
}
impl Display for DijkstraArc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.weight)
    }
}

impl DijkstraArc {
    pub fn new(weight: usize) -> Self {
        DijkstraArc { weight }
    }
}

impl Clone for DijkstraArc {
    fn clone(&self) -> Self {
        DijkstraArc {
            weight: self.weight,
        }
    }
}

pub fn dijkstra(network: &mut GraphNetwork<DijkstraNode, DijkstraArc>, start_node_id: NodeId) {
    // fill distance with infinity
    for node in &mut network.node_data {
        node.as_mut().unwrap().distance = Box::new(usize::MAX);
    }
    // set start node distance to 0
    network.mut_data_of_node(start_node_id).unwrap().distance = Box::new(0);

    // choices
    let mut heap = FibonacciHeap::<usize>::new();
    let mut heap_to_network = HashMap::<usize, usize>::new();

    for network_node_id in 0..network.node_data.len() {
        if let Some(network_node) = network.mut_data_of_node(network_node_id) {
            let heap_id = heap.push(*network_node.distance);
            heap_to_network.insert(heap_id, network_node_id);
            network_node.heap_id = heap_id;
        }
    }

    loop {
        // take closest node
        let minimum = heap.pop();
        if minimum.is_none() {
            break;
        }
        let (minimum_heap_id, _) = minimum.unwrap();
        let current_network_node_id = *heap_to_network.get(&minimum_heap_id).unwrap();
        let current_network_node_distance = *network
            .mut_data_of_node(current_network_node_id)
            .unwrap()
            .distance;

        let children: Vec<(NodeId, ArcId)> = network.from_node(current_network_node_id).collect();

        for (node_id, arc_id) in children.into_iter() {
            let arc = network.data_of_arc(arc_id).unwrap();
            let new_distance = current_network_node_distance + arc.weight;
            let node = network.mut_data_of_node(node_id).unwrap();
            if new_distance < *node.distance {
                *node.distance = new_distance;
                heap.modify(node.heap_id, new_distance);
            }
        }
    }
}

pub fn simple_dijkstra(
    network: &mut GraphNetwork<DijkstraNode, DijkstraArc>,
    start_node_id: NodeId,
) {
    // fill distance with infinity
    for node in &mut network.node_data {
        node.as_mut().unwrap().distance = Box::new(usize::MAX);
    }
    // set start node distance to 0
    network.mut_data_of_node(start_node_id).unwrap().distance = Box::new(0);

    // choices
    let mut unprocessed_nodes: Vec<usize> = (0..network.node_data.len()).collect();

    loop {
        // take closest node
        let mut current_node_id = None;
        let mut minimum_distance = usize::MAX;
        let mut new_unprocessed_nodes = Vec::new();
        for node_id in unprocessed_nodes.into_iter() {
            if let Some(node) = network.data_of_node(node_id) {
                if *node.distance < minimum_distance {
                    if let Some(current_node_id) = current_node_id {
                        new_unprocessed_nodes.push(current_node_id);
                    };
                    current_node_id = Some(node_id);
                    minimum_distance = *node.distance;
                } else {
                    new_unprocessed_nodes.push(node_id);
                }
            }
        }
        unprocessed_nodes = new_unprocessed_nodes;

        if current_node_id.is_none() {
            break;
        }

        let current_node_id = current_node_id.unwrap();
        let current_node_distance = *network.mut_data_of_node(current_node_id).unwrap().distance;

        let children: Vec<(NodeId, ArcId)> = network.from_node(current_node_id).collect();

        for (node_id, arc_id) in children.into_iter() {
            let arc = network.data_of_arc(arc_id).unwrap();
            let new_distance = current_node_distance + arc.weight;
            let node = network.mut_data_of_node(node_id).unwrap();
            if new_distance < *node.distance {
                *node.distance = new_distance;
            }
        }
    }
}

pub fn network_factory(
    arcs: Vec<(NodeId, NodeId, usize)>,
) -> GraphNetwork<DijkstraNode, DijkstraArc> {
    let mut network = GraphNetwork::<DijkstraNode, DijkstraArc>::new();
    let mut max_node_id: usize = 0;
    for (from, to, _) in arcs.iter() {
        max_node_id = max_node_id.max(*from).max(*to);
    }
    network.add_nodes(
        vec![
            DijkstraNode {
                distance: Box::new(usize::MAX),
                heap_id: usize::default()
            };
            max_node_id + 1
        ]
        .into_iter(),
    );
    network.bulk_connect(
        arcs.into_iter()
            .map(|(from, to, weight)| (from, to, DijkstraArc::new(weight))),
    );
    network
}

#[cfg(test)]
mod test {
    use super::*;

    fn mini_instance() -> GraphNetwork<DijkstraNode, DijkstraArc> {
        network_factory(vec![
            (0, 1, 1),
            (0, 2, 3),
            (0, 3, 2),
            (1, 2, 1),
            (3, 4, 2),
            (4, 3, 2),
            (4, 5, 2),
            (5, 3, 2),
        ])
    }

    #[test]
    fn test_dijkstra() {
        let mut network = mini_instance();
        dijkstra(&mut network, 0);
        println!("Network: \n{}", network);
    }

    #[test]
    fn test_simple_dijkstra() {
        let mut network = mini_instance();
        simple_dijkstra(&mut network, 0);
        println!("Network: {}", network);
    }
}
