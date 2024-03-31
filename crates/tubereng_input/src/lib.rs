#![warn(clippy::pedantic)]

#[derive(Debug, Clone, Copy)]
pub enum Input {
    KeyDown(keyboard::Key),
    KeyUp(keyboard::Key),
    MouseMotion((f64, f64)),
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

    pub fn on_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(key) => self.keyboard.on_key_down(key),
            Input::KeyUp(key) => self.keyboard.on_key_up(key),
            Input::MouseMotion(motion) => self.mouse.on_motion(motion),
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub mod mouse {
    pub struct State {
        last_motion: (f64, f64),
    }

    impl State {
        #[must_use]
        pub fn new() -> Self {
            Self {
                last_motion: (0.0, 0.0),
            }
        }

        #[must_use]
        pub fn motion(&self) -> &(f64, f64) {
            &self.last_motion
        }

        pub(crate) fn on_motion(&mut self, motion: (f64, f64)) {
            self.last_motion = motion;
        }

        pub(crate) fn clear_last_frame_inputs(&mut self) {
            self.last_motion = (0.0, 0.0);
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
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
        input.on_input(Input::KeyDown(Key::A));
        assert!(input.keyboard.is_key_down(Key::A));
    }
    #[test]
    fn input_state_on_key_up_changes_key_state() {
        let mut input = InputState::new();
        input.on_input(Input::KeyUp(Key::A));
        assert!(input.keyboard.is_key_up(Key::A));
        input.on_input(Input::KeyDown(Key::A));
        assert!(input.keyboard.is_key_down(Key::A));
    }
}
