use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub trait Node {
    fn mark_dirty(&mut self);
}

pub trait ChildNode {
    fn set_parent(&mut self, parent_node: Weak<RefCell<dyn Node>>);
}

type WeakNodeReference = Weak<RefCell<dyn Node>>;
type StrongChildReference = Rc<RefCell<dyn ChildNode>>;

pub struct Collection {
    // State whether the scene's state has changed
    dirty: bool,

    // The parent node (if present)
    parent_node: Option<WeakNodeReference>,

    // The list of child nodes
    child_nodes: Vec<StrongChildReference>,
}

impl Node for Collection {
    fn mark_dirty(&mut self) {
        self.dirty = true;
        match &self.parent_node {
            None => {
                // No parent node, so no need to mark it dirty
            }
            Some(weak) => {
                match weak.upgrade() {
                    None => panic!("Parent node was dropped"),
                    Some(rc) => {
                        rc.deref()
                            .borrow_mut()
                            .mark_dirty();
                    }
                }
            }
        }
    }
}

impl ChildNode for Collection {
    fn set_parent(&mut self, parent_node: WeakNodeReference) {
        self.parent_node = Some(parent_node);
    }
}

impl Collection {
    pub fn new() -> Collection {
        Collection { dirty: true, parent_node: None, child_nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: StrongChildReference) {
        self.child_nodes.push(node);
        self.mark_dirty();
    }
}

pub struct Scene {
    // State whether the scene's state has changed
    dirty: bool,

    // The root node of the scene
    root_node: StrongChildReference,
}

impl Node for Scene {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Scene {
    pub fn new() -> Rc<RefCell<Scene>> {
        let root_node = Rc::new(RefCell::new(Collection::new()));
        let scene = Rc::new(RefCell::new(Scene {
            dirty: false,
            root_node: root_node.clone(),
        }));
        let scene_weak = Rc::downgrade(&scene);
        root_node.borrow_mut().set_parent(scene_weak);
        return scene;
    }

    pub fn root_node(&self) -> StrongChildReference {
        return self.root_node.clone();
    }
}