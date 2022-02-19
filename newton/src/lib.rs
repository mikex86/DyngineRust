use rapier3d::math::{AngVector, Real};
use rapier3d::math::Vector;
use rapier3d::na::Vector3;
use rapier3d::prelude::{MassProperties, RigidBodyHandle as RapierRigidBodyHandle};
use rapier3d::prelude::RigidBodySet as RapierRigidBodySet;
use rapier3d::prelude::ColliderSet as RapierColliderSet;
use rapier3d::prelude::JointSet as RapierJointSet;
use rapier3d::prelude::PhysicsPipeline as RapierPhysicsPipeline;
use rapier3d::prelude::IslandManager as RapierIslandManager;
use rapier3d::prelude::BroadPhase as RapierBroadPhase;
use rapier3d::prelude::NarrowPhase as RapierNarrowPhase;
use rapier3d::prelude::CCDSolver as RapierCCDSolver;
use rapier3d::prelude::IntegrationParameters as RapierIntegrationParameters;
use rapier3d::prelude::RigidBodyType as RapierRigidBodyType;
use rapier3d::prelude::RigidBodyBuilder as RapierRigidBodyBuilder;
use rapier3d::prelude::ColliderBuilder as RapierColliderBuilder;

struct PhysicsWorld {
    rigid_body_set: RapierRigidBodySet,
    collider_set: RapierColliderSet,
    joint_set: RapierJointSet,
    gravity: Vector<Real>,
    physics_pipeline: RapierPhysicsPipeline,
    island_manager: RapierIslandManager,
    broad_phase: RapierBroadPhase,
    narrow_phase: RapierNarrowPhase,
    ccd_solver: RapierCCDSolver,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let rigid_body_set = RapierRigidBodySet::new();
        let collider_set = RapierColliderSet::new();
        let joint_set = RapierJointSet::new();

        let physics_pipeline = RapierPhysicsPipeline::new();
        let island_manager = RapierIslandManager::new();
        let broad_phase = RapierBroadPhase::new();
        let narrow_phase = RapierNarrowPhase::new();
        let ccd_solver = RapierCCDSolver::new();

        return PhysicsWorld {
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
    }
}

impl PhysicsWorld {
    pub fn step(&mut self, dt: f32) {
        let mut integration_parameters = RapierIntegrationParameters::default();
        integration_parameters.dt = dt;

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
}

trait Collider {}

struct CubeCollider {
    dimension: Vector3<f32>,
}

impl Collider for CubeCollider {}

struct SphereCollider {
    radius: f32,
}

impl Collider for SphereCollider {}

struct PhysicsObject {
    collider: Box<dyn Collider>,
    mesh_handle: RapierRigidBodyHandle,
}

impl PhysicsObject {

    pub fn new_box(
        physics_world: &mut PhysicsWorld,
        density: f32,
        dimension: Vector<f32>,
        position: Vector<f32>,
        orientation: AngVector<f32>,
        is_static: bool,
    ) -> Self {
        let rigid_body = RapierRigidBodyBuilder::new(if is_static { RapierRigidBodyType::Static } else { RapierRigidBodyType::Dynamic })
            .translation(Vector3::new(position.x, position.y, position.z))
            .rotation(orientation)
            .ccd_enabled(true)
            .build();

        let collider = RapierColliderBuilder::cuboid(dimension.x / 2.0, dimension.y / 2.0, dimension.z / 2.0)
            .restitution(0.7)
            .mass_properties(MassProperties::from_cuboid(density, dimension / 2.0))
            .build();

        let body_handle = physics_world.rigid_body_set.insert(rigid_body);
        physics_world.collider_set.insert_with_parent(collider, body_handle, &mut physics_world.rigid_body_set);

        return PhysicsObject {
            collider: Box::new(CubeCollider { dimension }),
            mesh_handle: body_handle,
        };
    }

    pub fn new_sphere(
        physics_world: &mut PhysicsWorld,
        density: f32,
        radius: f32,
        position: Vector<f32>,
        orientation: AngVector<f32>,
        is_static: bool,
    ) -> Self {
        let rigid_body = RapierRigidBodyBuilder::new(if is_static { RapierRigidBodyType::Static } else { RapierRigidBodyType::Dynamic })
            .translation(Vector3::new(position.x, position.y, position.z))
            .rotation(orientation)
            .ccd_enabled(true)
            .build();

        let collider = RapierColliderBuilder::ball(radius)
            .restitution(0.7)
            .mass_properties(MassProperties::from_ball(density, radius))
            .build();

        let body_handle = physics_world.rigid_body_set.insert(rigid_body);
        physics_world.collider_set.insert_with_parent(collider, body_handle, &mut physics_world.rigid_body_set);

        return PhysicsObject {
            collider: Box::new(SphereCollider { radius }),
            mesh_handle: body_handle,
        };
    }

}