use winit::event::KeyboardInput;

/// Types of objects who are interested in handling peripheral
/// source events should implement this. The object will receive
/// a call to handle an event if it is in "focus". See [winit]
/// crate's documentation for more information on events.
///
/// [winit]: https://docs.rs/winit/0.22.2/winit/event/index.html
pub trait EventHandler {
    fn handle_key(&mut self, key_input: KeyboardInput);
    fn handle_mouse_move(&mut self, dx: f32, dy: f32);
}
