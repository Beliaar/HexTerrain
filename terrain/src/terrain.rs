use std::collections::HashMap;

#[derive(Clone)]
pub struct Node {
    height: i32,
    nodes: Vec<usize>,
}

impl Node {
    pub fn new(height: i32) -> Node {
        Node {
            height,
            nodes: Vec::new(),
        }
    }

    pub fn zero() -> Node {
        Node {
            height: 0,
            nodes: Vec::new(),
        }
    }
}

pub struct Terrain<T: std::cmp::Eq + std::hash::Hash + Clone + Copy> {
    height_step: i32,
    node_map: HashMap<T, usize>,
    nodes: Vec<Node>,
}

impl<T: std::cmp::Eq + std::hash::Hash + Clone + Copy> Terrain<T> {
    pub fn new(height_step: i32) -> Terrain<T> {
        Terrain {
            height_step,
            node_map: HashMap::new(),
            nodes: Vec::new(),
        }
    }

    pub fn get_index_of_node(self, position: T) -> Option<usize> {
        match self.node_map.get(&position) {
            None => None,
            Some(index) => Some(*index),
        }
    }

    pub fn get_height_of_node(&self, position: T) -> Option<i32> {
        match self.node_map.get(&position) {
            None => None,
            Some(index) => Some(self.nodes[*index].height),
        }
    }

    /// Adds node to terrain if it does not already exist. Returns whether it was added or not.
    pub fn add_node(&mut self, position: T) -> bool {
        if self.node_map.contains_key(&position) {
            return false;
        }
        let node = Node::zero();
        let index = self.nodes.len();

        self.nodes.push(node);
        self.node_map.insert(position, index);

        true
    }

    /// Remove node from terrain if it exists. Returns whether it could be removed or not.
    pub fn remove_node(&mut self, position: T) -> bool {
        if self.node_map.contains_key(&position) {
            let index = self.node_map[&position];
            self.nodes.remove(index);
            self.node_map.remove(&position);
            return true;
        }
        false
    }

    /// Adds nodes that are connected. If either node is not present it will be created.
    pub fn add_connected_nodes(&mut self, first: T, second: T) {
        if !self.node_map.contains_key(&first) {
            self.add_node(first);
        }
        if !self.node_map.contains_key(&second) {
            self.add_node(second);
        }

        let first = self.node_map[&first];
        let second = self.node_map[&second];
        self.nodes[first].nodes.push(second);
        self.nodes[second].nodes.push(first);
    }

    pub fn increase_height(&mut self, node: T) {
        let index = self.node_map[&node];

        self.increase_height_recursive(index);
    }

    fn increase_height_recursive(&mut self, index: usize) {
        let mut node = &mut self.nodes[index];
        node.height += self.height_step;

        let node_height = node.height;
        for index in node.nodes.clone() {
            while self.nodes[index].height + self.height_step < node_height {
                self.increase_height_recursive(index);
            }
        }
    }

    pub fn decrease_height(&mut self, node: T) {
        let index = self.node_map[&node];

        self.decrease_height_recursive(index);
    }

    fn decrease_height_recursive(&mut self, index: usize) {
        let mut node = &mut self.nodes[index];
        node.height -= self.height_step;

        let node_height = node.height;
        for index in node.nodes.clone() {
            while self.nodes[index].height - self.height_step > node_height {
                self.decrease_height_recursive(index);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_node_adds_new_node_and_returns_true() {
        let mut terrain = Terrain::new(1);
        let return_value: bool = terrain.add_node(0);

        assert_eq!(true, return_value);
        assert_eq!(true, terrain.node_map.contains_key(&0));
        assert_eq!(0, terrain.nodes[0].height);
    }

    #[test]
    fn add_node_does_not_overwrite_existing_node_and_returns_false() {
        let mut terrain = Terrain::new(1);
        let mut node = Node::new(0);
        node.nodes.push(0);
        terrain.nodes.push(node);
        terrain.node_map.insert(0, 0);
        let return_value: bool = terrain.add_node(0);

        assert_eq!(false, return_value);
        assert_eq!(0, terrain.nodes[0].nodes[0]);
    }

    #[test]
    fn remove_node_removes_existing_node_and_returns_true() {
        let mut terrain = Terrain::new(1);
        terrain.nodes.push(Node::zero());
        terrain.node_map.insert(0, 0);
        let return_value: bool = terrain.remove_node(0);

        assert_eq!(true, return_value);
        assert_eq!(false, terrain.node_map.contains_key(&0));
        assert_eq!(true, terrain.nodes.is_empty())
    }

    #[test]
    fn remove_node_returns_false_if_node_does_not_exist() {
        let mut terrain = Terrain::new(1);
        let return_value: bool = terrain.remove_node(0);

        assert_eq!(false, return_value);
    }

    #[test]
    fn add_connected_nodes_connects_existing_nodes() {
        let mut terrain = Terrain::new(1);
        terrain.nodes.push(Node::new(0));
        let node1 = 0;
        terrain.node_map.insert(node1, 0);

        terrain.nodes.push(Node::new(0));
        let node2 = 1;
        terrain.node_map.insert(node2, 1);

        terrain.add_connected_nodes(node1, node2);

        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn add_connected_nodes_adds_and_connects_node_that_does_not_exist() {
        let mut terrain = Terrain::new(1);
        terrain.nodes.push(Node::new(0));
        let node1 = 0;

        terrain.node_map.insert(node1, 0);

        let node2 = 1;
        terrain.add_connected_nodes(node1, node2);

        assert_eq!(0, terrain.nodes[1].height);
        assert_eq!(1, terrain.node_map[&node2]);
        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn add_connected_nodes_adds_and_connects_nodes_that_do_not_exist() {
        let mut terrain = Terrain::new(1);
        let node1 = 0;
        let node2 = 1;

        terrain.add_connected_nodes(node1, node2);

        assert_eq!(0, terrain.nodes[0].height);
        assert_eq!(0, terrain.node_map[&node1]);
        assert_eq!(0, terrain.nodes[1].height);
        assert_eq!(1, terrain.node_map[&node2]);
        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn increase_height_increases_height_of_node_by_step() {
        let mut terrain = Terrain::new(1);
        terrain.nodes.push(Node::new(0));
        let node = 0;
        terrain.node_map.insert(node, 0);

        terrain.increase_height(node);

        assert_eq!(1, terrain.nodes[0].height);
        terrain.increase_height(node);
        assert_eq!(2, terrain.nodes[0].height);
    }

    #[test]
    fn increase_height_increases_height_of_connected_nodes() {
        // Setup is:
        // node: root node
        // connected_node_1: First node that is connected to node
        // connected_node_1_1: First node that is connected to connected_node_2
        // connected_node_2: Second node that is connected to node
        // connected_node_2_1: First node that is connected to connected_node_2

        let mut terrain = Terrain::new(1);

        terrain.nodes.push(Node::new(0));
        let node = 0;
        terrain.node_map.insert(node, 0);

        terrain.nodes.push(Node::new(0));
        let connected_node_1 = 1;
        terrain.node_map.insert(connected_node_1, 0);
        terrain.nodes[0].nodes.push(1);
        terrain.nodes[1].nodes.push(0);

        terrain.nodes.push(Node::new(2));
        let connected_node_1_1 = 2;
        terrain.node_map.insert(connected_node_1_1, 0);
        terrain.nodes[1].nodes.push(2);
        terrain.nodes[2].nodes.push(1);

        terrain.nodes.push(Node::new(0));
        let connected_node_2 = 3;
        terrain.node_map.insert(connected_node_2, 0);
        terrain.nodes[0].nodes.push(3);
        terrain.nodes[3].nodes.push(0);

        terrain.nodes.push(Node::new(0));
        let connected_node_2_1 = 4;
        terrain.node_map.insert(connected_node_2_1, 0);

        terrain.nodes[3].nodes.push(4);
        terrain.nodes[4].nodes.push(3);

        // 3 calls should result in the following
        // root node is increased to 3
        // Directly connected nodes are increased, or stay at 2 or higher
        // Nodes that are connected to directly connected nodes are increased or stay at 1 or higher

        terrain.increase_height(node);
        terrain.increase_height(node);
        terrain.increase_height(node);

        assert_eq!(3, terrain.nodes[0].height);
        assert_eq!(2, terrain.nodes[1].height);
        assert_eq!(2, terrain.nodes[2].height);
        assert_eq!(2, terrain.nodes[3].height);
        assert_eq!(1, terrain.nodes[4].height);
    }

    #[test]
    fn decrease_height_decreases_height_of_node_by_step() {
        let mut terrain = Terrain::new(1);
        terrain.nodes.push(Node::new(3));
        let node = 0;
        terrain.node_map.insert(node, 0);

        terrain.decrease_height(node);

        assert_eq!(2, terrain.nodes[0].height);
        terrain.decrease_height(node);
        assert_eq!(1, terrain.nodes[0].height);
    }

    #[test]
    fn decrease_height_decreases_height_of_connected_nodes() {
        // Setup is:
        // node: root node
        // connected_node_1: First node that is connected to node
        // connected_node_1_1: First node that is connected to connected_node_2
        // connected_node_2: Second node that is connected to node
        // connected_node_2_1: First node that is connected to connected_node_2

        let mut terrain = Terrain::new(1);

        terrain.nodes.push(Node::new(4));
        let node = 0;
        terrain.node_map.insert(node, 0);

        terrain.nodes.push(Node::new(3));
        let connected_node_1 = 1;
        terrain.node_map.insert(connected_node_1, 0);
        terrain.nodes[0].nodes.push(1);
        terrain.nodes[1].nodes.push(0);

        terrain.nodes.push(Node::new(2));
        let connected_node_1_1 = 2;
        terrain.node_map.insert(connected_node_1_1, 0);
        terrain.nodes[1].nodes.push(2);
        terrain.nodes[2].nodes.push(1);

        terrain.nodes.push(Node::new(4));
        let connected_node_2 = 3;
        terrain.node_map.insert(connected_node_2, 0);
        terrain.nodes[0].nodes.push(3);
        terrain.nodes[3].nodes.push(0);

        terrain.nodes.push(Node::new(3));
        let connected_node_2_1 = 4;
        terrain.node_map.insert(connected_node_2_1, 0);

        terrain.nodes[3].nodes.push(4);
        terrain.nodes[4].nodes.push(3);

        // 3 calls should result in the following
        // root node is decreased to 1
        // Directly connected nodes are decreased, or stay at 2 or higher
        // Nodes that are connected to directly connected nodes are decreased or stay at 3 or higher

        terrain.decrease_height(node);
        terrain.decrease_height(node);
        terrain.decrease_height(node);

        assert_eq!(1, terrain.nodes[0].height);
        assert_eq!(2, terrain.nodes[1].height);
        assert_eq!(2, terrain.nodes[2].height);
        assert_eq!(2, terrain.nodes[3].height);
        assert_eq!(3, terrain.nodes[4].height);
    }
}
