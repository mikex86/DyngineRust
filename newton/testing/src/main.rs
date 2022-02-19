extern crate kiss3d;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{Instant};
use glam::Vec3A;
use kiss3d::camera::{ArcBall};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Translation3};
use kiss3d::scene::SceneNode;
use kiss3d::window::{CanvasSetup, NumSamples, State, Window};
use obj::Obj;
use rapier3d::na::Point3;
use rapier3d::prelude::*;
use crate::nalgebra::Vector3;

trait Node {
    fn update(&mut self, rigid_body_set: &RigidBodySet, window: &mut Window);
}

struct Cube {
    cube_node: SceneNode,
    cube_handle: RigidBodyHandle,
}

impl Cube {
    fn new(rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet, window: &mut Window, position: Vec3A, dimension: Vec3A, is_static: bool) -> Cube {
        let rigid_body = RigidBodyBuilder::new(if is_static { RigidBodyType::Static } else { RigidBodyType::Dynamic })
            .translation(vector![position.x, position.y, position.z])
            .ccd_enabled(true)
            .build();
        let collider = ColliderBuilder::cuboid(dimension.x / 2.0, dimension.y / 2.0, dimension.z / 2.0).restitution(0.7).build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, rigid_body_set);

        let mut node = window.add_cube(dimension.x, dimension.y, dimension.z);
        node.set_visible(false);

        return Cube {
            cube_node: node,
            cube_handle: ball_body_handle,
        };
    }
}

impl Node for Cube {
    fn update(&mut self, rigid_body_set: &RigidBodySet, _: &mut Window) {
        let position = rigid_body_set[self.cube_handle].position().translation;

        self.cube_node.set_local_translation(Translation3::new(position.x, position.y, position.z));
        self.cube_node.set_visible(true);
    }
}

struct Mesh {
    mesh_node: SceneNode,
    mesh_handle: RigidBodyHandle,
}

impl Mesh {
    fn new(rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet, window: &mut Window, model_file: &Path, collider_model_file: &Path, position: Vec3A, is_static: bool) -> Mesh {
        let rigid_body = RigidBodyBuilder::new(if is_static { RigidBodyType::Static } else { RigidBodyType::Dynamic })
            .translation(vector![position.x, position.y, position.z])
            .ccd_enabled(false)
            .build();

        let input = BufReader::new(File::open(collider_model_file).unwrap());
        let obj_mesh: Obj = obj::load_obj(input).unwrap();

        let vertices: Vec<Point<Real>> = obj_mesh.vertices
            .iter()
            .map(|v| Point3::from(v.position))
            .collect();

        let indices: Vec<[u32; 3]> = obj_mesh.indices.chunks(3)
            .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
            .collect();

        let collider = ColliderBuilder::trimesh(vertices.clone(), indices.clone())
            .restitution(0.7)
            .mass_properties(MassProperties::from_convex_polyhedron(1.0, vertices.as_slice(), indices.as_slice()))
            .build();

        let body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, body_handle, rigid_body_set);

        let mut node = window.add_obj(Path::new(model_file), model_file.parent().unwrap(), Vector3::new(1.0, 1.0, 1.0));
        node.set_visible(false);

        return Mesh {
            mesh_node: node,
            mesh_handle: body_handle,
        };
    }
}

impl Node for Mesh {
    fn update(&mut self, rigid_body_set: &RigidBodySet, _: &mut Window) {
        let isometry = rigid_body_set[self.mesh_handle].position();
        let position = isometry.translation;
        let rotation = isometry.rotation;

        self.mesh_node.set_local_translation(Translation3::new(position.x, position.y, position.z));
        self.mesh_node.set_local_rotation(rotation);
        self.mesh_node.set_visible(true);
    }
}

struct AppState {
    last_time: std::time::Instant,
    nodes: Vec<Box<dyn Node>>,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    joint_set: JointSet,
    gravity: Vector<Real>,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
}

impl State for AppState {
    fn step(&mut self, window: &mut Window) {
        let delta_time_duration = Instant::now() - self.last_time;
        self.last_time = Instant::now();
        let delta_time_seconds = delta_time_duration.as_secs_f32();

        let fps = 1.0 / delta_time_seconds;
        if fps < 55.0 {
            println!("{} fps", fps);
        }

        let dt = 1.0 / 60.0;

        // println!("{}", delta_time_seconds);
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = dt;

        // Drive simulation
        {
            self.physics_pipeline.step(
                &self.gravity,
                &integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.joint_set,
                &mut self.ccd_solver,
                &(),
                &(),
            );
        }

        // Update nodes
        {
            for node in &mut self.nodes {
                node.update(&self.rigid_body_set, window);
            }
        }
    }
}

fn main() {
    let mut window = Window::new_with_setup("Testing", 900, 600, CanvasSetup { vsync: true, samples: NumSamples::Sixteen });
    window.set_light(Light::StickToCamera);

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    let joint_set = JointSet::new();

    let physics_pipeline = PhysicsPipeline::new();
    let island_manager = IslandManager::new();
    let broad_phase = BroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let ccd_solver = CCDSolver::new();
    let mut camera = ArcBall::new(Point3::from([15.0, 2.0, -5.0]), Point3::from([0.0, 1.0, -5.0]));

    let sim_size = 15;

    let mut nodes: Vec<Box<dyn Node>> = vec![
        // Box::new(Mesh::new(&mut rigid_body_set, &mut collider_set, &mut window, &Path::new("monkey.obj"), &Path::new("monkey_lowpoly.obj"), Vec3A::new(0.0, 5.0, 0.0), false)),
        // Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 8.0, 0.0), Vec3A::new(1.0, 1.0, 1.0), false)),
        // Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 12.0, -5.0), Vec3A::new(1.0, 1.0, 1.0), false)),

        // ground plane
        Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(sim_size as f32 * 10.0, 0.5, sim_size as f32 * 10.0), true)),
    ];

    for i in 0..2 * sim_size {
        for j in 0..2 * sim_size {
            let random_height = 2.0 + rand::random::<f32>() * sim_size as f32;
            nodes.push(Box::new(Mesh::new(&mut rigid_body_set, &mut collider_set, &mut window, &Path::new("monkey.obj"), &Path::new("monkey_lowpoly.obj"), Vec3A::new((i - sim_size) as f32 * 10.0, random_height, (j - sim_size) as f32 * 10.0), false)));
        }
    }

    let mut state = AppState {
        last_time: Instant::now(),
        nodes,
        rigid_body_set,
        collider_set,
        joint_set,
        gravity: Vector::new(0.0, -9.81, 0.0),
        physics_pipeline,
        island_manager,
        broad_phase,
        narrow_phase,
        ccd_solver,
    };

    while !window.should_close() {
        state.step(&mut window);
        window.render_with_camera(&mut camera);
    }
}