//! Components for input handling.

#[derive(Debug)]
pub struct Player;

#[derive(Debug)]
pub struct InputState {
    /// Really jank way of forcing the AIs to not update in real time.
    pub was_input_handled_this_frame: bool,
}

impl Default for InputState {
    fn default() -> InputState {
        InputState {
            was_input_handled_this_frame: false,
        }
    }
}