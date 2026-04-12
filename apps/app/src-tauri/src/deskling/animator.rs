use crate::config::{AnimationId, CompiledAnimation, CompiledConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTick {
    Playing,
    Finished,
}

#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    id: AnimationId,
    frame: usize,
    elapsed: f64,
    finished: bool,
}

impl AnimationPlayer {
    pub const fn new(id: AnimationId) -> Self {
        Self {
            id,
            frame: 0,
            elapsed: 0.0,
            finished: false,
        }
    }

    pub const fn id(&self) -> AnimationId {
        self.id
    }

    fn def<'a>(&self, cfg: &'a CompiledConfig) -> &'a CompiledAnimation {
        &cfg.animations[self.id.0]
    }

    pub fn current_sprite<'a>(&self, cfg: &'a CompiledConfig) -> &'a str {
        &self.def(cfg).frames[self.frame].sprite
    }

    pub fn tick(&mut self, dt: f64, cfg: &CompiledConfig) -> AnimationTick {
        if self.finished {
            return AnimationTick::Finished;
        }

        let def = self.def(cfg);
        self.elapsed += dt;

        while self.elapsed >= def.frames[self.frame].duration {
            self.elapsed -= def.frames[self.frame].duration;
            self.frame += 1;

            if self.frame >= def.frames.len() {
                if def.loops {
                    self.frame = 0;
                } else {
                    self.frame = def.frames.len() - 1;
                    self.finished = true;
                    return AnimationTick::Finished;
                }
            }
        }

        AnimationTick::Playing
    }
}
