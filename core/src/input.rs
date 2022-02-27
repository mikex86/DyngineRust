use std::collections::HashMap;
use winit::event::{DeviceId, ElementState, VirtualKeyCode};

pub(crate) struct DeviceInputHandler {
    key_states: HashMap<VirtualKeyCode, ElementState>,
}

impl DeviceInputHandler {
    pub(crate) fn new() -> Self {
        Self {
            key_states: HashMap::new(),
        }
    }
    pub(crate) fn get_key_state(&self, key: VirtualKeyCode) -> &ElementState {
        return self.key_states.get(&key).unwrap_or(&ElementState::Released);
    }

    pub(crate) fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        return self.get_key_state(key) == &ElementState::Pressed;
    }

    pub(crate) fn set_key_pressed(&mut self, key: VirtualKeyCode, pressed: ElementState) {
        self.key_states.insert(key, pressed);
    }
}

pub(crate) struct InputHandler {
    input_handlers: HashMap<DeviceId, DeviceInputHandler>,
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        Self {
            input_handlers: HashMap::new(),
        }
    }

    pub(crate) fn set_key_pressed(&mut self, device_id: DeviceId, key: VirtualKeyCode, pressed: ElementState) {
        self.input_handlers
            .entry(device_id)
            .or_insert(DeviceInputHandler::new())
            .set_key_pressed(key, pressed);
    }

    pub(crate) fn get_primary(&mut self) -> Option<&mut DeviceInputHandler> {
        return self.input_handlers.values_mut().next();
    }
}