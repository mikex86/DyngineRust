use Result;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use glam::{Vec3A, Quat};
use gltf::camera::Projection;
use wgpu::{Device, Queue, RenderPass};

use crate::camera::CameraNode;
use crate::camera::PerspectiveCamera;

pub trait RenderNode {
    /// Returns whether this node is currently "dirty" and needs to be updated.
    /// The dirty state indicates that the node's state has changed to such a degree it has visual impact when rendered.
    /// For example, a node's dirty state may be set to true when its transform changes, as the model matrix needs
    /// to be recalculated. But note, the render node implementation may still have internal dirty states
    /// to ensure most fine granular updating on the nodes state.
    /// Note that if any internal dirty state is set, the dirty state of the node must be true.
    /// Resolving the specific nature of the dirty state must be handled in [Self::resolve_dirty_state()].
    /// This is used to optimize the rendering process.
    fn is_dirty(&self) -> bool;

    /// Called to render the node. Performs the un-avoidable operations to render the node to the screen.
    /// [static_render_state] contains static render state
    /// [render_call_state] contains render state specific to this render call/frame
    fn render<'a, 'b: 'a>(&'b mut self, static_render_state: &mut StaticRenderState, render_call_state: &mut RenderCallState<'_, 'b>);

    /// Gets the render node out of the dirty state.
    /// Potentially expensive operation that rebuilds the resources affected by changed state of the node.
    /// [static_render_state] contains static render state
    fn resolve_dirty_state(&mut self, static_render_state: &mut StaticRenderState);

    /// Allows downcast of the render node to a concrete implementation.
    fn as_any(&self) -> &dyn Any;

    /// Allows mutable downcast of the render node to a concrete implementation.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct StaticRenderState {
    pub device: Rc<Device>,
    pub queue: Rc<Queue>,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

pub struct RenderCallState<'a, 'b: 'a> {
    pub render_pass: &'a mut RenderPass<'b>,
}

impl StaticRenderState {
    pub(crate) fn push_bind_group_layout(&mut self, bind_group: wgpu::BindGroupLayout) {
        self.bind_group_layouts.push(bind_group);
    }
}

pub struct RenderScene {
    pub nodes: HashMap<u64, Box<dyn RenderNode>>,
    pub static_render_state: StaticRenderState,
}

impl RenderScene {
    pub fn get_node_by_id<T: 'static>(&mut self, node_id: u64) -> Option<&mut T> where T: RenderNode {
        //let mut node_box: &mut Box<dyn RenderNode> = self.nodes.get_mut(&node_id).unwrap();
        //let z = node_box.as_any_mut();
        //z.downcast_mut()
        return self.nodes.get_mut(&node_id).map(|node| {
            let node = node.as_any_mut();
            return node.downcast_mut::<T>();
        }).flatten();
    }
}

#[derive(Debug)]
pub enum RenderSceneLoadingError {
    GltfError(gltf::Error),
}

impl RenderScene {
    pub fn new(static_render_state: StaticRenderState) -> Self {
        RenderScene { nodes: HashMap::new(), static_render_state }
    }

    pub fn load_gltf(static_render_state: StaticRenderState, gltf_bytes: &[u8]) -> Result<RenderScene, RenderSceneLoadingError> {
        match gltf::Gltf::from_slice(gltf_bytes) {
            Ok(render_scene) => {
                return RenderScene::parse_gltf(static_render_state, render_scene);
            }
            Err(error) => {
                return Err(RenderSceneLoadingError::GltfError(error));
            }
        }
    }

    fn parse_gltf(static_render_state: StaticRenderState, gltf_object: gltf::Gltf) -> Result<RenderScene, RenderSceneLoadingError> {
        let up_axis = Vec3A::new(0.0, 1.0, 0.0);
        let mut render_scene = RenderScene::new(static_render_state);
        for gltf_node in gltf_object.nodes() {
            let (translation, rotation, _scale) = gltf_node.transform().decomposed();
            let node_id = render_scene.nodes.len() as u64;

            // if child is camera
            let mut children = gltf_node.children();
            if children.len() == 1 {
                let camera_node = children.next().unwrap();
                match camera_node.camera() {
                    None => {}
                    Some(gltf_camera) => {
                        let camera_projection = gltf_camera.projection();
                        match camera_projection {
                            Projection::Perspective(gltf_perspective) => {
                                let fov = gltf_perspective.yfov().to_degrees();
                                let near = gltf_perspective.znear();
                                let far = gltf_perspective.zfar();
                                let rotation_quat = Quat::from_array(rotation);
                                let direction = -(rotation_quat * up_axis);
                                let camera = PerspectiveCamera::new(Vec3A::from(translation), Vec3A::from(direction), up_axis, fov, near, far, 1.0f32);
                                CameraNode::add_new(node_id, camera, &mut render_scene);
                            }
                            _ => {
                                panic!("Unsupported camera projection {:?}", camera_projection);
                            }
                        }
                    }
                }
            }
        }
        Ok(render_scene)
    }

    pub(crate) fn add_node(&mut self, node_id: u64, node: Box<dyn RenderNode>) {
        self.nodes.insert(node_id, node);
    }

    #[profiling::function]
    pub fn render<'a, 'b: 'a>(&'b mut self, render_call_state: &mut RenderCallState<'_, 'b>) {
        for (_, node) in &mut self.nodes {
            if node.is_dirty() {
                node.resolve_dirty_state(&mut self.static_render_state);
            }
            node.render(&mut self.static_render_state, render_call_state);
        }
    }
}