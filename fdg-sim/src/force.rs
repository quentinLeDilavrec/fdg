use std::ops::RangeInclusive;

use glam::Vec3;

use crate::{ForceGraph, Node};

/// A trait that contains all the methods that you need to create a force on the simulation.
pub trait Force<D: Clone> {
    /// Move the graph in any way you need.
    fn update(&self, graph: &mut ForceGraph<D>, dt: f32);
    /// Retrieve a mutable version of your internal dictionary that cooresponds to reused variables.
    fn dict_mut(&mut self) -> &mut Vec<(String, f32, RangeInclusive<f32>)>;
    /// Retrieve your internal dictionary that cooresponds to reused variables.
    fn dict(&self) -> &Vec<(String, f32, RangeInclusive<f32>)>;
    /// Reset your internal dictionary to the original settings.
    fn reset(&mut self);
}

#[derive(Clone)]
pub struct FruchtermanReingold {
    dict: Vec<(String, f32, RangeInclusive<f32>)>,
    dict_original: Vec<(String, f32, RangeInclusive<f32>)>
}

impl FruchtermanReingold {
    pub fn new<D: Clone>(scale: f32, cooloff_factor: f32) -> Self {
        let dict = vec![
            ("Scale".to_string(), scale, 0.1..=100.0),
            ("Cooloff Factor".to_string(), cooloff_factor, 0.0..=1.0),
        ];

        Self {
            dict: dict.clone(),
            dict_original: dict,
        }
    }

    fn repulsive_force<D>(&self, node_one: &Node<D>, node_two: &Node<D>) -> Vec3 {
        -((self.dict[0].1 * self.dict[0].1) / node_one.location.distance(node_two.location))
            * ((node_two.location - node_one.location)
                / node_one.location.distance(node_two.location))
    }

    fn attractive_force<D>(&self, node_one: &Node<D>, node_two: &Node<D>) -> Vec3 {
        (node_one.location.distance_squared(node_two.location) / self.dict[0].1)
            * ((node_two.location - node_one.location)
                / node_one.location.distance(node_two.location))
    }
}

impl<D: Clone> Force<D> for FruchtermanReingold {
    fn update(&self, graph: &mut ForceGraph<D>, dt: f32) {
        let graph_clone = graph.clone();

        for node_index in graph_clone.node_indices() {
            if graph_clone[node_index].locked {
                continue;
            }

            let mut final_force = Vec3::ZERO;

            for other_node_index in graph_clone.node_indices() {
                // skip duplicates
                if other_node_index == node_index {
                    continue;
                }

                let node = &graph[node_index];
                let other_node = &graph[other_node_index];

                final_force += self.repulsive_force(node, other_node);
            }

            for neighbor_index in graph.neighbors(node_index) {
                final_force += self.attractive_force(&graph[node_index], &graph[neighbor_index]);
            }

            let node = &mut graph[node_index];

            let acceleration = final_force / node.mass;
            node.velocity += acceleration * dt;
            node.velocity *= self.dict[1].1;

            node.location += node.velocity * dt;
        }
    }

    fn dict_mut(&mut self) -> &mut Vec<(String, f32, RangeInclusive<f32>)> {
        &mut self.dict
    }

    fn dict(&self) -> &Vec<(String, f32, RangeInclusive<f32>)> {
        &self.dict
    }

    fn reset(&mut self) {
        self.dict = self.dict_original.clone();
    }
}