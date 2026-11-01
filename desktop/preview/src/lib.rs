//! Explicit placeholder simulator, not osu!stable game logic.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scene {
    Canvas,
    Playground,
}
#[derive(Clone, Debug)]
pub struct PreviewState {
    pub paused: bool,
    pub scene: Scene,
    pub auto_play: bool,
    pub fullscreen: bool,
    micros: u64,
    accepted_inputs: u64,
}
impl Default for PreviewState {
    fn default() -> Self {
        Self {
            paused: true,
            scene: Scene::Canvas,
            auto_play: true,
            fullscreen: false,
            micros: 0,
            accepted_inputs: 0,
        }
    }
}
impl PreviewState {
    pub fn advance(&mut self, elapsed_micros: u64) {
        if !self.paused {
            self.micros = self.micros.saturating_add(elapsed_micros.min(100_000));
        }
    }
    pub fn input(&mut self, interaction_tool: bool, canvas_focused: bool) -> bool {
        let accept =
            !self.paused && interaction_tool && canvas_focused && self.scene == Scene::Playground;
        if accept {
            self.accepted_inputs += 1;
        }
        accept
    }
    pub fn micros(&self) -> u64 {
        self.micros
    }
    pub fn accepted_inputs(&self) -> u64 {
        self.accepted_inputs
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn freeze_drops_input_and_resume_does_not_accumulate_elapsed_time() {
        let mut p = PreviewState {
            scene: Scene::Playground,
            ..Default::default()
        };
        p.advance(5000);
        assert!(!p.input(true, true));
        assert_eq!(p.micros(), 0);
        p.paused = false;
        p.advance(1000);
        assert_eq!(p.micros(), 1000);
        assert!(p.input(true, true));
        assert!(!p.input(false, true));
        assert!(!p.input(true, false));
        p.paused = true;
        p.advance(10000);
        p.paused = false;
        p.advance(1000);
        assert_eq!(p.micros(), 2000);
        assert_eq!(p.accepted_inputs(), 1);
    }
}
