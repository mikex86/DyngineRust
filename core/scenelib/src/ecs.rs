use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::f32::consts::PI;
use glam::{DQuat, EulerRot, Quat, Vec3, Vec3A, Vec4};
use specs::{Component, VecStorage, HashMapStorage, NullStorage, Entity, World, WorldExt, Builder, WriteStorage, ReadStorage, System, Read, Join, ParJoin, DispatcherBuilder, Dispatcher};
use specs::prelude::ParallelIterator;
use crate::camera::{CameraRenderNode, PerspectiveCamera};
use crate::scene::{RenderNodeHandle, RenderScene};

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct PositionComponent {
    pub position: Vec3A,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct VelocityComponent {
    pub velocity: Vec3A,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct RotationComponent {
    /// [yaw], [pitch] and [roll] determine [quaternion]
    /// unit: radians
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,

    pub quaternion: Quat,
}

#[derive(Default)]
struct DeltaTimeResource(pub f32);

struct NewtonianExplicitIntegratorSystem;

impl<'a> System<'a> for NewtonianExplicitIntegratorSystem {
    type SystemData = (Read<'a, DeltaTimeResource>,
                       ReadStorage<'a, VelocityComponent>,
                       WriteStorage<'a, PositionComponent>);

    fn run(&mut self, (delta_time, velocities, mut positions): Self::SystemData) {
        (&velocities, &mut positions)
            .par_join()
            .for_each(|(velocity, position)| {
                position.position += velocity.velocity * delta_time.0;
            });
    }
}

pub type ECSEntityHandle = u64;

pub struct ECSWorld {
    world: World,
    ecs_entities: HashMap<ECSEntityHandle, Box<dyn ECSEntity>>,
    camera_handles: Vec<ECSEntityHandle>,
    next_entiy_handle: ECSEntityHandle,
    dispatcher: Dispatcher<'static, 'static>,
}


#[derive(Default, Debug, Clone)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub sprinting: bool,
    pub should_roll: bool,

    pub delta_yaw: f32,
    pub delta_pitch: f32,
}


impl MovementInput {
    pub fn new() -> Self {
        return MovementInput {
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            sprinting: false,
            should_roll: false,

            delta_yaw: 0.0,
            delta_pitch: 0.0,
        };
    }

    /// Clears all movement input. Prepares for the next frame.
    pub fn new_frame(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
        self.sprinting = false;
        self.delta_yaw = 0.0;
        self.delta_pitch = 0.0;
    }
}

impl ECSWorld {
    pub fn new() -> ECSWorld {
        let mut world = World::new();

        world.insert(DeltaTimeResource(0.0));
        world.insert(MovementInputResource::new());

        world.register::<PositionComponent>();
        world.register::<VelocityComponent>();

        let dispatcher = DispatcherBuilder::new()
            .with(FlyingCameraSystem, "flying_camera_system", &[])
            .with(NewtonianExplicitIntegratorSystem, "position_integrator", &["flying_camera_system"])
            .build();

        return ECSWorld {
            world,
            ecs_entities: HashMap::new(),
            camera_handles: Vec::new(),
            next_entiy_handle: 1,
            dispatcher,
        };
    }

    pub fn update(&mut self, delta_time: f64, movement_input: MovementInput, render_scene: &mut RenderScene) {
        // Update delta time resource
        {
            let mut delta = self.world.write_resource::<DeltaTimeResource>();
            *delta = DeltaTimeResource(delta_time as f32);
        }

        // Update movement input resource
        {
            let mut movement_input_resource = self.world.write_resource::<MovementInputResource>();
            *movement_input_resource = MovementInputResource { movement_input };
        }
        // Update ECS
        {
            self.dispatcher.dispatch(&self.world);
            self.world.maintain();
        }
        // Update scene
        {
            for entity in &mut self.ecs_entities.values_mut() {
                entity.update_render_node(&self.world, render_scene);
            }
        }
    }

    pub fn add_entity<T: ECSEntity + 'static>(&mut self, entity: Box<T>) -> ECSEntityHandle {
        // if camera, add to camera list
        if TypeId::of::<T>() == TypeId::of::<CameraEntity>() {
            self.camera_handles.push(self.next_entiy_handle);
        }
        // add to ecs
        let entity_handle = self.next_entiy_handle;
        self.ecs_entities.insert(entity_handle, entity);
        self.next_entiy_handle += 1;
        return entity_handle;
    }

    pub fn get_entity(&self, entity_handle: &ECSEntityHandle) -> Option<&Box<dyn ECSEntity>> {
        return self.ecs_entities.get(entity_handle);
    }

    pub fn get_cameras(&self) -> &Vec<ECSEntityHandle> {
        return &self.camera_handles;
    }

    pub fn get_primary_camera(&self) -> Option<ECSEntityHandle> {
        return self.camera_handles.first().copied();
    }
}


#[derive(Component, Debug)]
#[storage(HashMapStorage)]
struct CameraComponent {
    // The camera's forward axis (not to be confused with the camera's foward vector)
    forward_axis: Vec3A,
    // The camera's up axis (not to be confused with the camera's up vector)
    up_axis: Vec3A,
    // The camera's vertical field of view.
    fov: f32,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct FlyingCameraComponent;

pub trait ECSEntity {
    fn update_render_node(&mut self, world: &World, render_scene: &mut RenderScene);

    fn get_render_node(&self) -> Option<&RenderNodeHandle>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct CameraEntity {
    camera_render_node_handle: RenderNodeHandle,
    specs_entity_handle: Entity,
}

impl ECSEntity for CameraEntity {
    fn update_render_node(&mut self, world: &World, render_scene: &mut RenderScene) {
        let position_component = world.read_component::<PositionComponent>();
        let position = position_component.get(self.specs_entity_handle).unwrap().position;

        let rotation_component = world.read_component::<RotationComponent>();
        let rotation = rotation_component.get(self.specs_entity_handle).unwrap().quaternion;

        let camera_component = world.read_component::<CameraComponent>();
        let camera_data = camera_component.get(self.specs_entity_handle).unwrap();

        let render_node = render_scene.nodes.get_mut(&self.camera_render_node_handle).unwrap().as_any_mut();
        let camera_render_node = render_node.downcast_mut::<CameraRenderNode>().unwrap();

        camera_render_node.set_position(position);
        camera_render_node.set_rotation(Quat::from_xyzw(rotation.x as f32, rotation.y as f32, rotation.z as f32, rotation.w as f32));
        camera_render_node.set_fov(camera_data.fov);
    }

    fn get_render_node(&self) -> Option<&RenderNodeHandle> {
        return Some(&self.camera_render_node_handle);
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        return self;
    }
}

impl CameraEntity {
    pub fn add_flying(ecs_word: &mut ECSWorld, render_scene: &mut RenderScene,
                      position: Vec3A,
                      direction: Vec3A,
                      forward_axis: Vec3A,
                      up_axis: Vec3A,
                      fov: f32,
                      near: f32,
                      far: Option<f32>,
                      aspect: f32) -> ECSEntityHandle {
        let world = &mut ecs_word.world;
        world.register::<PositionComponent>();
        world.register::<VelocityComponent>();
        world.register::<RotationComponent>();
        world.register::<CameraComponent>();
        world.register::<FlyingCameraComponent>();

        let rotation;
        {
            let dot = direction.dot(forward_axis);
            if (dot - 1.0).abs() < f32::EPSILON {
                rotation = Quat::IDENTITY;
            } else if (dot + 1.0).abs() < f32::EPSILON {
                rotation = Quat::from_axis_angle(Vec3::from(up_axis), PI);
            } else {
                let angle = dot.acos();
                let rot_axis = forward_axis.cross(direction).normalize();
                rotation = Quat::from_axis_angle(Vec3::from(rot_axis), angle);
            }
        }

        let (yaw, pitch, roll) = rotation.to_euler(EulerRot::YXZ);

        let entity = world.create_entity()
            .with(PositionComponent { position: position })
            .with(VelocityComponent { velocity: Vec3A::ZERO })
            .with(RotationComponent { quaternion: rotation, yaw: yaw, pitch: pitch, roll: roll })
            .with(CameraComponent { forward_axis, up_axis, fov })
            .with(FlyingCameraComponent)
            .build();

        let camera = PerspectiveCamera::new(position, direction, forward_axis, up_axis, fov, near, far, aspect);
        let camera_node_handle = CameraRenderNode::add_new(camera, render_scene);
        let camera_entity = CameraEntity { camera_render_node_handle: camera_node_handle, specs_entity_handle: entity };
        return ecs_word.add_entity(Box::new(camera_entity));
    }
}

#[derive(Default)]
struct MovementInputResource {
    movement_input: MovementInput,
}

impl MovementInputResource {
    pub fn new() -> Self {
        return MovementInputResource { movement_input: MovementInput::new() };
    }
}

struct FlyingCameraSystem;

impl<'a> System<'a> for FlyingCameraSystem {
    type SystemData = (
        Read<'a, DeltaTimeResource>,
        Read<'a, MovementInputResource>,
        WriteStorage<'a, RotationComponent>,
        WriteStorage<'a, VelocityComponent>,
        WriteStorage<'a, CameraComponent>,
        ReadStorage<'a, FlyingCameraComponent>,
    );

    fn run(&mut self, (delta_time_resource, movement_input_resource, mut rotations, mut velocities, cameras, flying_cameras): Self::SystemData) {
        let movement_input = &movement_input_resource.movement_input;
        let delta_yaw = movement_input.delta_yaw;
        let delta_pitch = movement_input.delta_pitch;
        for (rotation, velocity, camera, _) in (&mut rotations, &mut velocities, &cameras, &flying_cameras).join() {
            // Rotate the camera.
            {
                let rot_quat = &mut rotation.quaternion;

                if !movement_input.should_roll {
                    rotation.yaw += delta_yaw;
                    rotation.pitch += delta_pitch;
                } else {
                    rotation.roll += delta_yaw;
                }
                *rot_quat = Quat::from_euler(EulerRot::ZYX, rotation.roll, rotation.yaw, rotation.pitch);
            }

            // Add movement input to velocity
            {
                let forward = rotation.quaternion * camera.forward_axis;
                let up = rotation.quaternion * camera.up_axis;
                let right = up.cross(forward);

                let mut move_dir = Vec3A::ZERO;
                {
                    if movement_input.forward ^ movement_input.backward {
                        move_dir += if movement_input.forward {
                            forward
                        } else {
                            -forward
                        };
                    }
                    if movement_input.left ^ movement_input.right {
                        move_dir += if movement_input.right {
                            right
                        } else {
                            -right
                        };
                    }
                    if movement_input.up ^ movement_input.down {
                        move_dir += if movement_input.up {
                            up
                        } else {
                            -up
                        };
                    }
                }
                velocity.velocity = move_dir * (if movement_input.sprinting { 2.0 } else { 1.0 });
            }
        }
    }
}

