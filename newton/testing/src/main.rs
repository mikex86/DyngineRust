extern crate kiss3d;

use std::time::Instant;
use glam::Vec3A;
use kiss3d::light::Light;
use kiss3d::nalgebra::Translation3;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use rapier3d::prelude::*;

trait Node {
    fn update(&mut self, rigid_body_set: &RigidBodySet, window: &mut Window);
}

struct Cube {
    cube_node: SceneNode,
    cube_handle: RigidBodyHandle,
}

impl Cube {
    fn new(rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet, window: &mut Window, position: Vec3A, is_static: bool) -> Cube {
        let rigid_body = RigidBodyBuilder::new(if is_static { RigidBodyType::Static } else { RigidBodyType::Dynamic })
            .translation(vector![position.x, position.y, position.z])
            .ccd_enabled(true)
            .build();
        let collider = ColliderBuilder::cuboid(1.0, 1.0, 1.0).restitution(0.7).build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, rigid_body_set);

        let mut node = window.add_cube(2.0, 2.0, 2.0);
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

struct AppState {
    last_frame_end: std::time::Instant,
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
        let delta_time_duration = Instant::now() - self.last_frame_end;
        let delta_time_seconds = delta_time_duration.as_secs_f32();
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = delta_time_seconds;

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
        self.last_frame_end = Instant::now();
    }
}

fn main() {
    let mut window = Window::new("Newton (testing)");
    window.set_light(Light::StickToCamera);

    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    let joint_set = JointSet::new();

    let physics_pipeline = PhysicsPipeline::new();
    let island_manager = IslandManager::new();
    let broad_phase = BroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let ccd_solver = CCDSolver::new();


    let state = AppState {
        last_frame_end: Instant::now(),
        nodes: vec![
            Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 9.0, 0.0), false)),
            Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 7.0, 0.0), false)),
            Box::new(Cube::new(&mut rigid_body_set, &mut collider_set, &mut window, Vec3A::new(0.0, 1.0, 0.0), true)),
        ],
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
    window.render_loop(state)
}