use euclid::{UnknownUnit, Vector2D};
use gdnative::api::GlobalConstants;
use gdnative::api::Node as GodotNode;
use gdnative::api::{
    ArrayMesh, CollisionShape, Mesh, MeshInstance, SphereShape, StaticBody, SurfaceTool,
};
use gdnative::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use terrain::terrain::Terrain;

type Vector2Di32 = Vector2D<i32, UnknownUnit>;
type HexagonData = (Hexagon, HashMap<Vector2Di32, Vector2>, Vec<TerrainNode>);
type NodeData = (Vector2Di32, u32);

const LEFT: Vector2Di32 = Vector2Di32::new(-2, 0);
const TOP_LEFT: Vector2Di32 = Vector2Di32::new(-1, -2);
const TOP_RIGHT: Vector2Di32 = Vector2Di32::new(1, -2);
const RIGHT: Vector2Di32 = Vector2Di32::new(2, 0);
const BOTTOM_RIGHT: Vector2Di32 = Vector2Di32::new(1, 2);
const BOTTOM_LEFT: Vector2Di32 = Vector2Di32::new(-1, 2);

struct Hexagon {
    center: Vector2Di32,
    left: Vector2Di32,
    top_left: Vector2Di32,
    top_right: Vector2Di32,
    right: Vector2Di32,
    bottom_right: Vector2Di32,
    bottom_left: Vector2Di32,
}

impl Hexagon {
    pub fn new(center: Vector2Di32) -> Hexagon {
        let left = center + LEFT;
        let top_left = center + TOP_LEFT;
        let top_right = center + TOP_RIGHT;
        let right = center + RIGHT;
        let bottom_right = center + BOTTOM_RIGHT;
        let bottom_left = center + BOTTOM_LEFT;

        Hexagon {
            center,
            left,
            top_left,
            top_right,
            right,
            bottom_right,
            bottom_left,
        }
    }
}

#[derive(Clone)]
struct TerrainNode {
    key: Vector2Di32,
    connections: Vec<Vector2Di32>,
    uv: Vector2,
}

impl TerrainNode {
    pub fn new(key: Vector2Di32, uv: Vector2) -> TerrainNode {
        TerrainNode {
            key,
            connections: Vec::new(),
            uv,
        }
    }
}

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct HexTerrain {
    nodes: Vec<TerrainNode>,
    hexagon_map: HashMap<Vector2Di32, Hexagon>,
    vertex_map: HashMap<Vector2Di32, Vector2>,
    terrain: Terrain<Vector2Di32>,
    #[property]
    hex_radius: f32,
    #[property]
    field_radius: u32,
    #[property]
    node_height: f32,
}

#[methods]
impl HexTerrain {
    pub fn new(_owner: TRef<'_, Spatial>) -> Self {
        Self {
            nodes: Vec::new(),
            hexagon_map: HashMap::new(),
            vertex_map: HashMap::new(),
            terrain: Terrain::new(1),
            hex_radius: 0.5,
            field_radius: 0,
            node_height: 0.5,
        }
    }

    #[export]
    pub fn _input(&mut self, owner: TRef<'_, Spatial>, event: Variant) {
        if let Some(event) = event.try_to_object::<InputEventKey>() {
            let event = unsafe { event.assume_safe() };
            if event.is_pressed() {
                let scancode = event.scancode();
                if scancode == GlobalConstants::KEY_PLUS || scancode == GlobalConstants::KEY_KP_ADD
                {
                    self.field_radius += 1;
                    self.terrain = Terrain::new(1);
                    self.create_hex_nodes();
                }
                if (scancode == GlobalConstants::KEY_MINUS
                    || scancode == GlobalConstants::KEY_KP_SUBTRACT)
                    && self.field_radius > 0
                {
                    self.field_radius -= 1;
                    self.terrain = Terrain::new(1);
                    self.create_hex_nodes();
                }

                self.update_vertices(owner);
            }
        }
    }

    #[export]
    pub fn node_increase(&mut self, owner: TRef<'_, Spatial>, x: i64, y: i64) {
        let clicked_node = Vector2Di32::new(x as i32, y as i32);
        self.terrain.increase_height(clicked_node);
        self.update_vertices(owner);
    }

    #[export]
    pub fn node_decrease(&mut self, owner: TRef<'_, Spatial>, x: i64, y: i64) {
        let clicked_node = Vector2Di32::new(x as i32, y as i32);
        self.terrain.decrease_height(clicked_node);
        self.update_vertices(owner);
    }

    #[export]
    pub fn _ready(&mut self, owner: TRef<'_, Spatial>) {
        self.create_hex_nodes();
        self.update_vertices(owner);
    }

    fn update_vertices(&mut self, owner: TRef<'_, Spatial>) {
        let surface_tool_hex = SurfaceTool::new();
        let surface_tool_grid = SurfaceTool::new();

        surface_tool_hex.begin(Mesh::PRIMITIVE_TRIANGLES);

        let mut processed_indicators = HashSet::<Vector2Di32>::new();

        let resource_loader = ResourceLoader::godot_singleton();
        let indicator_node = resource_loader
            .load("res://Indicator.tscn", "PackedScene", false)
            .unwrap()
            .cast::<PackedScene>()
            .unwrap();
        let indicator_mesh: TRef<'_, PackedScene> = unsafe { indicator_node.assume_safe() };

        let indicator_mesh = unsafe { indicator_mesh.instance(0).unwrap().assume_safe() };
        let indicator_mesh: TRef<'_, StaticBody> = indicator_mesh.cast::<StaticBody>().unwrap();
        let collision = indicator_mesh.get_node("Collision").unwrap();
        let collision = unsafe { collision.assume_safe() };
        let collision: TRef<'_, CollisionShape> = collision.cast::<CollisionShape>().unwrap();

        let shape = SphereShape::new();
        shape.set_radius(self.hex_radius.into());
        shape.set_margin(5.0);

        collision.set_shape(shape);

        let nodes_node = unsafe { owner.get_node("Nodes").unwrap().assume_safe() };

        for child in nodes_node.get_children().iter() {
            let child = child.try_to_object::<GodotNode>().unwrap();
            nodes_node.remove_child(child);
            unsafe { child.assume_safe().queue_free() };
        }

        for node_data in self.nodes.clone() {
            for connection in node_data.connections {
                self.terrain.add_connected_nodes(node_data.key, connection);
            }

            let height: i32 = match self.terrain.get_height_of_node(node_data.key) {
                None => panic!(),
                Some(height) => height,
            };

            let vector_data = self.vertex_map[&node_data.key];

            let vertex = Vector3::new(
                vector_data.x,
                height as f32 * self.node_height,
                vector_data.y,
            );

            let uv = node_data.uv;
            surface_tool_hex.add_uv(uv);
            surface_tool_hex.add_vertex(vertex);

            if !processed_indicators.contains(&node_data.key) {
                let new_indicator = unsafe {
                    indicator_mesh
                        .duplicate(Node::DUPLICATE_USE_INSTANCING)
                        .unwrap()
                        .assume_safe()
                };
                let new_indicator: TRef<'_, StaticBody> =
                    new_indicator.cast::<StaticBody>().unwrap();
                new_indicator.set_translation(vertex);

                let signal_data = VariantArray::new();
                signal_data.push(node_data.key.x);
                signal_data.push(node_data.key.y);

                new_indicator
                    .connect(
                        "increase",
                        owner,
                        "node_increase",
                        signal_data.duplicate().into_shared(),
                        0,
                    )
                    .unwrap();
                new_indicator
                    .connect(
                        "decrease",
                        owner,
                        "node_decrease",
                        signal_data.duplicate().into_shared(),
                        0,
                    )
                    .unwrap();

                nodes_node.add_child(new_indicator, false);

                processed_indicators.insert(node_data.key);
            }
        }

        let mut tmp_mesh = ArrayMesh::new();
        surface_tool_hex.generate_normals(false);
        tmp_mesh = match surface_tool_hex.commit(tmp_mesh, Mesh::ARRAY_COMPRESS_DEFAULT) {
            None => return,
            Some(mesh) => unsafe { mesh.assume_unique() },
        };

        let mesh_instance = owner
            .get_node("HexMesh")
            .and_then(|node| unsafe { node.assume_safe_if_sane() })
            .and_then(|node| node.cast::<MeshInstance>());
        match mesh_instance {
            None => {}
            Some(mesh_instance) => {
                mesh_instance.set_mesh(tmp_mesh);
            }
        }

        let grid_node = owner
            .get_node("Grid")
            .and_then(|node| unsafe { node.assume_safe_if_sane() });
        let grid_node: TRef<'_, GodotNode> = match grid_node {
            None => panic!(),
            Some(grid_node) => grid_node,
        };

        for child in grid_node.get_children().iter() {
            let child: Variant = child;
            let child = child.try_to_object::<GodotNode>().unwrap();
            let child = unsafe { child.assume_safe() };
            grid_node.remove_child(child);
            child.queue_free();
        }
        let line_height = 0.01;

        for hexagon in self.hexagon_map.values() {
            let mut grid_mesh = ArrayMesh::new();
            surface_tool_grid.begin(Mesh::PRIMITIVE_LINE_LOOP);

            let key = hexagon.left;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            let key = hexagon.top_left;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            let key = hexagon.top_right;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            let key = hexagon.right;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            let key = hexagon.bottom_right;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            let key = hexagon.bottom_left;
            let vertex = self.vertex_map[&key];
            let vertex_height =
                self.terrain.get_height_of_node(key).unwrap() as f32 * self.node_height;
            let vertex = Vector3::new(vertex.x, vertex_height + line_height, vertex.y);
            surface_tool_grid.add_vertex(vertex);

            grid_mesh = match surface_tool_grid.commit(grid_mesh, Mesh::ARRAY_COMPRESS_DEFAULT) {
                None => {
                    godot_error!("Could not commit grid mesh");
                    return;
                }
                Some(mesh) => unsafe { mesh.assume_unique() },
            };
            let mesh_instance = MeshInstance::new();

            mesh_instance.set_mesh(grid_mesh);

            grid_node.add_child(mesh_instance, false);
        }
    }

    fn create_hex_nodes(&mut self) {
        let (vertex_data_sender, vertex_data_receiver): (
            Sender<HexagonData>,
            Receiver<HexagonData>,
        ) = mpsc::channel();
        let (node_sender, node_receiver): (Sender<NodeData>, Receiver<NodeData>) = mpsc::channel();
        let mut nodes_data = Vec::<TerrainNode>::new();
        let mut hexagons = HashMap::<Vector2Di32, Hexagon>::new();
        let mut vertices_data = HashMap::<Vector2Di32, Vector2>::new();

        let mut threads = Vec::new();

        let radius = self.field_radius;
        let hex_radius = self.hex_radius;
        let mut processed_nodes = HashSet::new();
        let mut finished_threads = 0;

        processed_nodes.insert(Vector2Di32::zero());

        {
            let vertex_data_sender = vertex_data_sender.clone();
            let node_sender = node_sender.clone();
            threads.push(thread::spawn(move || {
                Self::create_hex_vertices(
                    Vector2Di32::zero(),
                    radius,
                    hex_radius,
                    vertex_data_sender,
                    node_sender,
                );
            }));
        }

        while processed_nodes.len() != finished_threads {
            let mut received = true;
            while received {
                match node_receiver.try_recv() {
                    Ok(node) => {
                        if !processed_nodes.contains(&node.0) {
                            processed_nodes.insert(node.0);
                            let vertex_data_sender = vertex_data_sender.clone();
                            let node_sender = node_sender.clone();
                            threads.push(thread::spawn(move || {
                                Self::create_hex_vertices(
                                    node.0,
                                    node.1,
                                    hex_radius,
                                    vertex_data_sender,
                                    node_sender,
                                );
                            }));
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        received = false;
                    }
                }
            }

            received = true;

            while received {
                match vertex_data_receiver.try_recv() {
                    Ok(mut vertex_data) => {
                        hexagons.insert(vertex_data.0.center, vertex_data.0);
                        vertices_data.extend(vertex_data.1);
                        nodes_data.append(&mut vertex_data.2);
                        finished_threads += 1;
                    }
                    Err(_) => {
                        received = false;
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            //godot_print!("{}-{}", threads.len(), finished_threads);
            thread::sleep(Duration::from_millis(10));
        }
        self.nodes = nodes_data;
        self.hexagon_map = hexagons;
        self.vertex_map = vertices_data;
    }

    fn create_hex_vertices(
        center: Vector2Di32,
        radius: u32,
        hex_radius: f32,
        vertex_data_sender: Sender<HexagonData>,
        node_sender: Sender<NodeData>,
    ) {
        let left = center + LEFT;
        let top_left = center + TOP_LEFT;
        let top_right = center + TOP_RIGHT;
        let right = center + RIGHT;
        let bottom_right = center + BOTTOM_RIGHT;
        let bottom_left = center + BOTTOM_LEFT;

        let mut hexagon = Hexagon::new(center);
        hexagon.left = left;
        hexagon.top_left = top_left;
        hexagon.top_right = top_right;
        hexagon.right = right;
        hexagon.bottom_right = bottom_right;
        hexagon.bottom_left = bottom_left;

        if radius > 0 {
            node_sender.send((left + TOP_LEFT, radius - 1)).unwrap();
            node_sender
                .send((top_left + TOP_RIGHT, radius - 1))
                .unwrap();
            node_sender.send((top_right + RIGHT, radius - 1)).unwrap();
            node_sender
                .send((right + BOTTOM_RIGHT, radius - 1))
                .unwrap();
            node_sender
                .send((bottom_right + BOTTOM_LEFT, radius - 1))
                .unwrap();
            node_sender.send((bottom_left + LEFT, radius - 1)).unwrap();
        }

        let mut vertices_data = HashMap::<Vector2Di32, Vector2>::new();

        vertices_data.insert(
            center,
            Vector2::new(center.x as f32 * hex_radius, center.y as f32 * hex_radius),
        );
        let mut center_node_data = TerrainNode::new(center, Vector2::new(0.5, 0.5));
        center_node_data.connections.push(left);
        center_node_data.connections.push(top_left);
        center_node_data.connections.push(top_right);
        center_node_data.connections.push(right);
        center_node_data.connections.push(bottom_right);
        center_node_data.connections.push(bottom_left);

        vertices_data.insert(
            left,
            Vector2::new(left.x as f32 * hex_radius, left.y as f32 * hex_radius),
        );
        let mut left_data = TerrainNode::new(left, Vector2::new(0.0, 0.5));
        left_data.connections.push(top_left);
        left_data.connections.push(bottom_left);

        vertices_data.insert(
            top_left,
            Vector2::new(
                top_left.x as f32 * hex_radius,
                top_left.y as f32 * hex_radius,
            ),
        );
        let mut top_left_data = TerrainNode::new(top_left, Vector2::new(0.25, 0.0));
        top_left_data.connections.push(left);
        top_left_data.connections.push(top_right);

        vertices_data.insert(
            top_right,
            Vector2::new(
                top_right.x as f32 * hex_radius,
                top_right.y as f32 * hex_radius,
            ),
        );
        let mut top_right_data = TerrainNode::new(top_right, Vector2::new(0.75, 0.00));
        top_right_data.connections.push(top_left);
        top_right_data.connections.push(right);

        vertices_data.insert(
            right,
            Vector2::new(right.x as f32 * hex_radius, right.y as f32 * hex_radius),
        );
        let mut right_data = TerrainNode::new(right, Vector2::new(1.0, 0.5));
        right_data.connections.push(top_right);
        right_data.connections.push(bottom_right);

        vertices_data.insert(
            bottom_right,
            Vector2::new(
                bottom_right.x as f32 * hex_radius,
                bottom_right.y as f32 * hex_radius,
            ),
        );
        let mut bottom_right_data = TerrainNode::new(bottom_right, Vector2::new(0.75, 1.0));
        bottom_right_data.connections.push(right);
        bottom_right_data.connections.push(bottom_left);

        vertices_data.insert(
            bottom_left,
            Vector2::new(
                bottom_left.x as f32 * hex_radius,
                bottom_left.y as f32 * hex_radius,
            ),
        );
        let mut bottom_left_data = TerrainNode::new(bottom_left, Vector2::new(0.25, 1.0));
        bottom_left_data.connections.push(bottom_right);
        bottom_left_data.connections.push(left);

        let mut nodes_data = Vec::<TerrainNode>::new();
        nodes_data.push(center_node_data.clone());
        nodes_data.push(left_data.clone());
        nodes_data.push(top_left_data.clone());

        nodes_data.push(center_node_data.clone());
        nodes_data.push(top_left_data);
        nodes_data.push(top_right_data.clone());

        nodes_data.push(center_node_data.clone());
        nodes_data.push(top_right_data);
        nodes_data.push(right_data.clone());

        nodes_data.push(center_node_data.clone());
        nodes_data.push(right_data);
        nodes_data.push(bottom_right_data.clone());

        nodes_data.push(center_node_data.clone());
        nodes_data.push(bottom_right_data);
        nodes_data.push(bottom_left_data.clone());

        nodes_data.push(center_node_data);
        nodes_data.push(bottom_left_data);
        nodes_data.push(left_data);

        match vertex_data_sender.send((hexagon, vertices_data, nodes_data)) {
            Ok(_) => {}
            Err(err) => godot_print!("Could not send vertex data: {}", err),
        };
    }
}

#[cfg(test)]
mod tests {}
