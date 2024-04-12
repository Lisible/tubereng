#![warn(clippy::pedantic)]

#[derive(Debug, Clone, Copy)]
pub enum Input {
    MouseButtonDown(mouse::Button),
    MouseButtonUp(mouse::Button),
    KeyDown(keyboard::Key),
    KeyUp(keyboard::Key),
    MouseMotion((f64, f64)),
    CursorMoved((f64, f64)),
}

pub struct InputState {
    pub keyboard: keyboard::State,
    pub mouse: mouse::State,
}

impl InputState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyboard: keyboard::State::new(),
            mouse: mouse::State::new(),
        }
    }

    pub fn clear_last_frame_inputs(&mut self) {
        self.mouse.clear_last_frame_inputs();
        self.keyboard.clear_last_frame_inputs();
    }

    pub fn on_input(&mut self, input: &Input) {
        match input {
            Input::MouseButtonDown(button) => self.mouse.on_button_down(*button),
            Input::MouseButtonUp(button) => self.mouse.on_button_up(*button),
            Input::KeyDown(key) => self.keyboard.on_key_down(*key),
            Input::KeyUp(key) => self.keyboard.on_key_up(*key),
            Input::MouseMotion(motion) => self.mouse.on_motion(*motion),
            Input::CursorMoved(position) => self.mouse.on_move(*position),
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub mod mouse {
    use log::trace;

    #[derive(Default, Debug, Clone, Copy)]
    pub(crate) struct ButtonState {
        pub current: bool,
        pub previous: bool,
    }

    pub struct State {
        pub(super) button_state: [ButtonState; BUTTON_COUNT],
        last_motion: (f64, f64),
        position: (f64, f64),
    }

    impl State {
        #[must_use]
        pub fn new() -> Self {
            Self {
                button_state: [ButtonState::default(); BUTTON_COUNT],
                last_motion: (0.0, 0.0),
                position: (0.0, 0.0),
            }
        }

        #[must_use]
        pub fn motion(&self) -> &(f64, f64) {
            &self.last_motion
        }

        #[must_use]
        pub fn position(&self) -> &(f64, f64) {
            &self.position
        }

        pub(crate) fn on_motion(&mut self, motion: (f64, f64)) {
            self.last_motion = motion;
        }

        pub(crate) fn on_move(&mut self, position: (f64, f64)) {
            self.position = position;
        }

        #[must_use]
        pub fn is_button_down(&self, button: Button) -> bool {
            self.button_state[button as usize].current
        }

        #[must_use]
        pub fn was_button_down(&self, button: Button) -> bool {
            self.button_state[button as usize].previous
        }

        #[must_use]
        pub fn is_button_up(&self, button: Button) -> bool {
            !self.button_state[button as usize].current
        }

        pub(crate) fn on_button_up(&mut self, button: Button) {
            trace!("Button up: {button:?}");
            self.button_state[button as usize].current = false;
        }

        pub(crate) fn on_button_down(&mut self, button: Button) {
            trace!("Button down: {button:?}");
            self.button_state[button as usize].current = true;
        }

        pub(crate) fn clear_last_frame_inputs(&mut self) {
            self.last_motion = (0.0, 0.0);
            for button_state in &mut self.button_state {
                button_state.previous = button_state.current;
            }
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
    }

    const BUTTON_COUNT: usize = 4;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Button {
        Left = 0,
        Middle,
        Right,
        Unknown,
    }
}

pub mod keyboard {
    use log::trace;

    #[derive(Debug, Default, Copy, Clone)]
    pub(crate) struct KeyState {
        pub current: bool,
        pub previous: bool,
    }

    pub struct State {
        pub(super) key_state: [KeyState; KEY_COUNT],
    }

    impl State {
        #[must_use]
        pub fn new() -> Self {
            Self {
                key_state: [KeyState::default(); KEY_COUNT],
            }
        }

        pub fn clear_last_frame_inputs(&mut self) {
            for key_state in &mut self.key_state {
                key_state.previous = key_state.current;
            }
        }

        #[must_use]
        pub fn is_key_down(&self, key: Key) -> bool {
            self.key_state[key as usize].current
        }

        #[must_use]
        pub fn was_key_down(&self, key: Key) -> bool {
            self.key_state[key as usize].previous
        }

        #[must_use]
        pub fn is_key_up(&self, key: Key) -> bool {
            !self.key_state[key as usize].current
        }

        pub(crate) fn on_key_up(&mut self, key: Key) {
            trace!("Key up: {key:?}");
            self.key_state[key as usize].current = false;
        }

        pub(crate) fn on_key_down(&mut self, key: Key) {
            trace!("Key down: {key:?}");
            self.key_state[key as usize].current = true;
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
    }

    // TODO:
    // Use https://doc.rust-lang.org/std/mem/fn.variant_count.html when it stabilizes
    // In the meantime a proc_macro could be made to generate this constant.
    const KEY_COUNT: usize = 39;
    #[derive(Debug, Copy, Clone)]
    pub enum Key {
        Escape = 0,
        Return,
        LShift,
        RShift,
        LControl,
        RControl,
        Backspace,
        Space,
        ArrowUp,
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        Unknown,
    }

    pub enum Modifier {
        Shift,
        LControl,
        RControl,
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::Key;

    use super::*;

    #[test]
    fn input_state_initial_key_state_is_false() {
        let input = InputState::new();
        assert!(!input.keyboard.is_key_down(Key::A));
    }

    #[test]
    fn input_state_check_key_down_when_key_is_down() {
        let mut input = InputState::new();
        input.keyboard.key_state[Key::Escape as usize].current = true;
        assert!(input.keyboard.is_key_down(Key::Escape));
    }

    #[test]
    fn input_state_on_key_down_changes_key_state() {
        let mut input = InputState::new();
        assert!(input.keyboard.is_key_up(Key::A));
        input.on_input(&Input::KeyDown(Key::A));
        assert!(input.keyboard.is_key_down(Key::A));
    }
    #[test]
    fn input_state_on_key_up_changes_key_state() {
        let mut input = InputState::new();
        input.on_input(&Input::KeyUp(Key::A));
        assert!(input.keyboard.is_key_up(Key::A));
        input.on_input(&Input::KeyDown(Key::A));
        assert!(input.keyboard.is_key_down(Key::A));
    }
}
