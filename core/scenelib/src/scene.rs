use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

use wgpu::{Device, Queue, RenderPass};
use crate::camera::CameraRenderNode;
use crate::ecs::CameraEntity;

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
    /// This is called when the node is marked as dirty BUT this does NOT mean
    /// the render node's dirty state is garanteed to be resolved in the next frame.
    /// Eg. the dirty state is not resolved when the not is not visible. (TODO: this is not implemented yet)
    fn resolve_dirty_state(&mut self, static_render_state: &mut StaticRenderState);

    /// Allows downcast of the render node to a concrete implementation.
    fn as_any(&self) -> &dyn Any;

    /// Allows mutable downcast of the render node to a concrete implementation.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A type used to reference a render node in the scene.
pub type RenderNodeHandle = u64;

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
    next_handle: RenderNodeHandle,
    pub nodes: HashMap<RenderNodeHandle, Box<dyn RenderNode>>,
    cameras: Vec<RenderNodeHandle>,
    pub static_render_state: StaticRenderState,
}

impl RenderScene {
    pub fn get_node_by_id<T: 'static>(&mut self, node_handle: &RenderNodeHandle) -> Option<&mut T> where T: RenderNode {
        return self.nodes.get_mut(node_handle).map(|node| {
            let node = node.as_any_mut();
            return node.downcast_mut::<T>();
        }).flatten();
    }
}

impl RenderScene {
    pub fn new(static_render_state: StaticRenderState) -> Self {
        RenderScene { next_handle: 1, nodes: HashMap::new(), cameras: Vec::new(), static_render_state }
    }

    pub(crate) fn add_node<T: RenderNode + 'static>(&mut self, node: Box<T>) -> RenderNodeHandle {
        if TypeId::of::<T>() != TypeId::of::<CameraRenderNode>() {
            self.cameras.push(self.next_handle);
        }
        let handle = self.next_handle;
        self.nodes.insert(handle, node);
        self.next_handle += 1;
        return handle;
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

    pub fn set_active_camera(&mut self, camera_handle: &RenderNodeHandle) {
        {
            let camera: &mut CameraRenderNode = self.get_node_by_id(camera_handle).unwrap();
            camera.set_active();
        }

        let cameras = self.cameras.clone();
        for camera_handle in &cameras {
            if camera_handle != camera_handle {
                let camera: &mut CameraRenderNode = self.get_node_by_id(camera_handle).unwrap();
                camera.set_inactive();
            }
        }
    }
}