//! Independently authored synthetic probe of the editor's preview contract.
//! No stable rules, source, assets, audio backend, UI, or project implementation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Edit,
    Interact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    First,
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Conflict,
    NonMonotonicTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub simulation_ms: u64,
    /// A logical cursor only: this probe does not play audio.
    pub audio_cursor_ms: u64,
    pub paused: bool,
    pub content_revision: u64,
    pub accepted_game_inputs: u64,
    pub scene: Scene,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    generation: u64,
    content_revision: u64,
    simulation_ms: u64,
    scene: Scene,
}

pub struct RenderRequest {
    stamp: Stamp,
    material: [u8; 3],
}

pub struct Frame {
    stamp: Stamp,
    rgb: Vec<u8>,
}

impl Frame {
    pub fn rgb(&self) -> &[u8] {
        &self.rgb
    }
}

impl RenderRequest {
    /// Synthetic 32x32 RGB pattern; no game coordinates or compositing rules.
    pub fn render(self) -> Frame {
        let mut rgb = self.material.repeat(32 * 32);
        let x = (self.stamp.simulation_ms % 32) as usize;
        let y = if self.stamp.scene == Scene::First {
            8
        } else {
            24
        };
        let offset = (y * 32 + x) * 3;
        rgb[offset..offset + 3].copy_from_slice(&[255, 255, 255]);
        Frame {
            stamp: self.stamp,
            rgb,
        }
    }
}

pub struct Probe {
    observation: Observation,
    wall_anchor_ms: u64,
    tool: Tool,
    focused: bool,
    material: [u8; 3],
    generation: u64,
    presented: Option<Stamp>,
}

impl Probe {
    pub fn new(wall_ms: u64) -> Self {
        Self {
            observation: Observation {
                simulation_ms: 0,
                audio_cursor_ms: 0,
                paused: false,
                content_revision: 0,
                accepted_game_inputs: 0,
                scene: Scene::First,
            },
            wall_anchor_ms: wall_ms,
            tool: Tool::Edit,
            focused: true,
            material: [40, 80, 120],
            generation: 0,
            presented: None,
        }
    }

    pub fn observe(&self) -> Observation {
        self.observation
    }

    pub fn tick(&mut self, wall_ms: u64) -> Result<(), Error> {
        let elapsed = wall_ms
            .checked_sub(self.wall_anchor_ms)
            .ok_or(Error::NonMonotonicTime)?;
        self.wall_anchor_ms = wall_ms;
        if !self.observation.paused && elapsed > 0 {
            self.observation.simulation_ms = self.observation.simulation_ms.saturating_add(elapsed);
            self.observation.audio_cursor_ms = self.observation.simulation_ms;
            self.generation += 1;
        }
        Ok(())
    }

    pub fn pause(&mut self, paused: bool, wall_ms: u64) -> Result<(), Error> {
        // Anchor even if no render/update happened during a long pause.
        self.tick(wall_ms)?;
        if self.observation.paused != paused {
            self.observation.paused = paused;
            self.generation += 1;
        }
        Ok(())
    }

    /// The only content mutation in this synthetic application facade.
    pub fn replace_material(
        &mut self,
        expected_revision: u64,
        material: [u8; 3],
    ) -> Result<bool, Error> {
        if expected_revision != self.observation.content_revision {
            return Err(Error::Conflict);
        }
        if self.material == material {
            return Ok(false);
        }
        self.material = material;
        self.observation.content_revision += 1;
        self.generation += 1;
        Ok(true)
    }

    pub fn select_scene(&mut self, scene: Scene) {
        if self.observation.scene != scene {
            self.observation.scene = scene;
            self.generation += 1;
        }
    }

    pub fn select_tool(&mut self, tool: Tool) {
        self.tool = tool;
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Synthetic button presses are accepted or discarded immediately, never queued.
    pub fn game_press(&mut self) -> bool {
        let accepted = !self.observation.paused && self.focused && self.tool == Tool::Interact;
        if accepted {
            self.observation.accepted_game_inputs += 1;
        }
        accepted
    }

    fn stamp(&self) -> Stamp {
        Stamp {
            generation: self.generation,
            content_revision: self.observation.content_revision,
            simulation_ms: self.observation.simulation_ms,
            scene: self.observation.scene,
        }
    }

    pub fn render_request(&self) -> Option<RenderRequest> {
        let stamp = self.stamp();
        if self.presented == Some(stamp) {
            None
        } else {
            Some(RenderRequest {
                stamp,
                material: self.material,
            })
        }
    }

    pub fn present(&mut self, frame: &Frame) -> bool {
        if frame.stamp != self.stamp() || self.presented == Some(frame.stamp) {
            return false;
        }
        self.presented = Some(frame.stamp);
        true
    }
}
