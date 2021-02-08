use euclid::default::Vector2D;
use euclid::default::Vector3D;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

type Vector2 = Vector2D<f32>;
type OrderedVector2 = Vector2D<OrderedFloat<f32>>;
type Vector3 = Vector3D<f32>;

#[derive(Clone)]
pub struct Node {
    height: f32,
    nodes: Vec<usize>,
}

impl Node {
    pub fn new(height: f32) -> Node {
        Node {
            height,
            nodes: Vec::new(),
        }
    }

    pub fn zero() -> Node {
        Node {
            height: 0.0,
            nodes: Vec::new(),
        }
    }
}

pub struct Terrain {
    height_step: f32,
    node_map: HashMap<OrderedVector2, usize>,
    nodes: Vec<Node>,
}

impl Terrain {
    pub fn new(height_step: f32) -> Terrain {
        Terrain {
            height_step,
            node_map: HashMap::new(),
            nodes: Vec::new(),
        }
    }

    /// Adds node to terrain if it does not already exist. Returns whether it was added or not.
    pub fn add_node(&mut self, position: Vector2) -> bool {
        let position = OrderedVector2::new(
            OrderedFloat::from(position.x),
            OrderedFloat::from(position.y),
        );
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
    pub fn remove_node(&mut self, position: Vector2) -> bool {
        let position = OrderedVector2::new(
            OrderedFloat::from(position.x),
            OrderedFloat::from(position.y),
        );
        if self.node_map.contains_key(&position) {
            let index = self.node_map[&position];
            self.nodes.remove(index);
            self.node_map.remove(&position);
            return true;
        }
        false
    }

    /// Adds nodes that are connected. If either node is not present it will be created.
    pub fn add_connected_nodes(&mut self, first: Vector2, second: Vector2) {
        let first_key =
            OrderedVector2::new(OrderedFloat::from(first.x), OrderedFloat::from(first.y));
        let second_key =
            OrderedVector2::new(OrderedFloat::from(second.x), OrderedFloat::from(second.y));

        if !self.node_map.contains_key(&first_key) {
            self.add_node(first);
        }
        if !self.node_map.contains_key(&second_key) {
            self.add_node(second);
        }

        let first = self.node_map[&first_key];
        let second = self.node_map[&second_key];
        self.nodes[first].nodes.push(second);
        self.nodes[second].nodes.push(first);
    }

    pub fn increase_height(&mut self, node: Vector2) {
        let node = OrderedVector2::new(OrderedFloat::from(node.x), OrderedFloat::from(node.y));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_node_adds_new_node_and_returns_true() {
        let mut terrain = Terrain::new(0.1);
        let return_value: bool = terrain.add_node(Vector2::zero());

        assert_eq!(true, return_value);
        assert_eq!(true, terrain.node_map.contains_key(&OrderedVector2::zero()));
        assert_eq!(0.0, terrain.nodes[0].height);
    }

    #[test]
    fn add_node_does_not_overwrite_existing_node_and_returns_false() {
        let mut terrain = Terrain::new(0.1);
        let mut node = Node::new(0.0);
        node.nodes.push(0);
        terrain.nodes.push(node);
        terrain.node_map.insert(OrderedVector2::zero(), 0);
        let return_value: bool = terrain.add_node(Vector2::zero());

        assert_eq!(false, return_value);
        assert_eq!(0, terrain.nodes[0].nodes[0]);
    }

    #[test]
    fn remove_node_removes_existing_node_and_returns_true() {
        let mut terrain = Terrain::new(0.1);
        terrain.nodes.push(Node::zero());
        terrain.node_map.insert(OrderedVector2::zero(), 0);
        let return_value: bool = terrain.remove_node(Vector2::zero());

        assert_eq!(true, return_value);
        assert_eq!(
            false,
            terrain.node_map.contains_key(&OrderedVector2::zero())
        );
        assert_eq!(true, terrain.nodes.is_empty())
    }

    #[test]
    fn remove_node_returns_false_if_node_does_not_exist() {
        let mut terrain = Terrain::new(0.1);
        let return_value: bool = terrain.remove_node(Vector2::zero());

        assert_eq!(false, return_value);
    }

    #[test]
    fn add_connected_nodes_connects_existing_nodes() {
        let mut terrain = Terrain::new(0.1);
        terrain.nodes.push(Node::new(0.0));
        let node1 = Vector2::new(1.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(OrderedFloat::from(node1.x), OrderedFloat::from(node1.y)),
            0,
        );

        terrain.nodes.push(Node::new(0.0));
        let node2 = Vector2::new(1.0, 1.0);
        terrain.node_map.insert(
            OrderedVector2::new(OrderedFloat::from(node2.x), OrderedFloat::from(node2.y)),
            1,
        );

        terrain.add_connected_nodes(node1, node2);

        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn add_connected_nodes_adds_and_connects_node_that_does_not_exist() {
        let mut terrain = Terrain::new(0.1);
        terrain.nodes.push(Node::new(0.0));
        let node1 = Vector2::new(1.0, 0.0);

        terrain.node_map.insert(
            OrderedVector2::new(OrderedFloat::from(node1.x), OrderedFloat::from(node1.y)),
            0,
        );

        let node2 = Vector2::new(1.0, 1.0);
        terrain.add_connected_nodes(node1, node2);

        assert_eq!(0.0, terrain.nodes[1].height);
        assert_eq!(
            1,
            terrain.node_map
                [&OrderedVector2::new(OrderedFloat::from(node2.x), OrderedFloat::from(node2.y))]
        );
        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn add_connected_nodes_adds_and_connects_nodes_that_do_not_exist() {
        let mut terrain = Terrain::new(0.1);
        let node1 = Vector2::new(1.0, 0.0);
        let node2 = Vector2::new(1.0, 1.0);

        terrain.add_connected_nodes(node1, node2);

        assert_eq!(0.0, terrain.nodes[0].height);
        assert_eq!(
            0,
            terrain.node_map
                [&OrderedVector2::new(OrderedFloat::from(node1.x), OrderedFloat::from(node1.y))]
        );
        assert_eq!(0.0, terrain.nodes[1].height);
        assert_eq!(
            1,
            terrain.node_map
                [&OrderedVector2::new(OrderedFloat::from(node2.x), OrderedFloat::from(node2.y))]
        );
        assert_eq!(1, terrain.nodes[0].nodes.len());
        assert_eq!(1, terrain.nodes[1].nodes.len());
        assert_eq!(1, terrain.nodes[0].nodes[0]);
        assert_eq!(0, terrain.nodes[1].nodes[0]);
    }

    #[test]
    fn increase_height_increases_height_of_node_by_step() {
        let mut terrain = Terrain::new(0.1);
        terrain.nodes.push(Node::new(0.0));
        let node1 = Vector2::new(1.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(OrderedFloat::from(node1.x), OrderedFloat::from(node1.y)),
            0,
        );

        terrain.increase_height(node1);

        assert_eq!(0.1, terrain.nodes[0].height);
        terrain.increase_height(node1);
        assert_eq!(0.2, terrain.nodes[0].height);
    }

    #[test]
    fn increase_height_increases_height_of_connected_nodes() {
        // Setup is:
        // node: root node
        // connected_node_1: First node that is connected to node
        // connected_node_1_1: First node that is connected to connected_node_2
        // connected_node_2: Second node that is connected to node
        // connected_node_2_1: First node that is connected to connected_node_2

        let mut terrain = Terrain::new(0.1);

        terrain.nodes.push(Node::new(0.0));
        let node = Vector2::new(0.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(OrderedFloat::from(node.x), OrderedFloat::from(node.y)),
            0,
        );

        terrain.nodes.push(Node::new(0.0));
        let connected_node_1 = Vector2::new(1.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(
                OrderedFloat::from(connected_node_1.x),
                OrderedFloat::from(connected_node_1.y),
            ),
            0,
        );
        terrain.nodes[0].nodes.push(1);
        terrain.nodes[1].nodes.push(0);

        terrain.nodes.push(Node::new(0.2));
        let connected_node_1_1 = Vector2::new(1.0, 1.0);
        terrain.node_map.insert(
            OrderedVector2::new(
                OrderedFloat::from(connected_node_1_1.x),
                OrderedFloat::from(connected_node_1_1.y),
            ),
            0,
        );
        terrain.nodes[1].nodes.push(2);
        terrain.nodes[2].nodes.push(1);

        terrain.nodes.push(Node::new(0.0));
        let connected_node_2 = Vector2::new(2.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(
                OrderedFloat::from(connected_node_2.x),
                OrderedFloat::from(connected_node_2.y),
            ),
            0,
        );
        terrain.nodes[0].nodes.push(3);
        terrain.nodes[3].nodes.push(0);

        terrain.nodes.push(Node::new(0.0));
        let connected_node_2_1 = Vector2::new(2.0, 0.0);
        terrain.node_map.insert(
            OrderedVector2::new(
                OrderedFloat::from(connected_node_2_1.x),
                OrderedFloat::from(connected_node_2_1.y),
            ),
            0,
        );
        terrain.nodes[3].nodes.push(4);
        terrain.nodes[4].nodes.push(3);

        // 3 calls should result in the following
        // root node is increased to 0.3
        // Directly connected nodes are increased, or stay at 0.2 or higher
        // Nodes that are connected to directly connected nodes are increased or stay at 0.1 or higher

        terrain.increase_height(node);
        terrain.increase_height(node);
        terrain.increase_height(node);

        assert_eq!(0.3, terrain.nodes[0].height);
        assert_eq!(0.2, terrain.nodes[1].height);
        assert_eq!(0.2, terrain.nodes[2].height);
        assert_eq!(0.2, terrain.nodes[3].height);
        assert_eq!(0.1, terrain.nodes[4].height);
    }
}
