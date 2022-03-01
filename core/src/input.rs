use std::collections::HashMap;
use winit::event::{DeviceId, ElementState, VirtualKeyCode};

pub(crate) struct KeyboardInputHandler {
    key_states: HashMap<VirtualKeyCode, ElementState>,
}

impl KeyboardInputHandler {
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
    keyboard_input_handlers: HashMap<DeviceId, KeyboardInputHandler>,
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        Self {
            keyboard_input_handlers: HashMap::new(),
        }
    }

    pub(crate) fn set_key_pressed(&mut self, device_id: DeviceId, key: VirtualKeyCode, pressed: ElementState) {
        self.keyboard_input_handlers
            .entry(device_id)
            .or_insert_with(|| KeyboardInputHandler::new())
            .set_key_pressed(key, pressed);
    }

    pub(crate) fn get_primary_keyboard(&mut self) -> Option<&mut KeyboardInputHandler> {
        return self.keyboard_input_handlers.values_mut().next();
    }
}