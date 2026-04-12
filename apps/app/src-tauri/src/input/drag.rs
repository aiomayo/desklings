use crate::config::CompiledDrag;
use crate::deskling::state::{DesklingPhysics, Facing};

#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct DragOutcome {
    pub vx: f64,
    pub vy: f64,
}

const MAX_SAMPLES: usize = 8;
const MIN_DT: f64 = 0.001;

#[derive(Debug)]
pub struct DragTracker {
    offset_x: f64,
    offset_y: f64,
    samples: Vec<(f64, f64, f64)>,
    smooth_dx: f64,
    smooth_dy: f64,
    facing: Facing,
}

impl Default for DragTracker {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            samples: Vec::with_capacity(MAX_SAMPLES),
            smooth_dx: 0.0,
            smooth_dy: 0.0,
            facing: Facing::Right,
        }
    }
}

impl DragTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(&mut self, deskling: &DesklingPhysics, cx: f64, cy: f64, now: f64) {
        self.offset_x = cx - deskling.x;
        self.offset_y = cy - deskling.y;
        self.samples.clear();
        self.samples.push((cx, cy, now));
        self.smooth_dx = 0.0;
        self.smooth_dy = 0.0;
        self.facing = Facing::Right;
    }

    pub fn update(
        &mut self,
        deskling: &mut DesklingPhysics,
        cfg: &CompiledDrag,
        cx: f64,
        cy: f64,
        now: f64,
    ) {
        let bounds = deskling.virtual_bounds;
        deskling.x = (cx - self.offset_x).clamp(bounds.min_x, bounds.max_x - deskling.size);
        deskling.y = (cy - self.offset_y).clamp(bounds.min_y, bounds.max_y - deskling.size);

        self.samples.push((cx, cy, now));
        if self.samples.len() > MAX_SAMPLES {
            self.samples.remove(0);
        }

        if self.samples.len() >= 2 {
            let prev = self.samples[self.samples.len() - 2];
            let last = self.samples[self.samples.len() - 1];
            let dt = (last.2 - prev.2).max(MIN_DT);
            let raw_dx = (last.0 - prev.0) / dt;
            let raw_dy = (last.1 - prev.1) / dt;

            let alpha = cfg.smoothing_alpha;
            self.smooth_dx += alpha * (raw_dx - self.smooth_dx);
            self.smooth_dy += alpha * (raw_dy - self.smooth_dy);
        }

        if self.smooth_dx > cfg.direction_hysteresis {
            self.facing = Facing::Right;
        } else if self.smooth_dx < -cfg.direction_hysteresis {
            self.facing = Facing::Left;
        }
        deskling.facing = self.facing;
    }

    pub fn end(&mut self, max_v: f64) -> DragOutcome {
        let mut vx = 0.0;
        let mut vy = 0.0;
        if self.samples.len() >= 2 {
            let first = self.samples[0];
            let last = self.samples[self.samples.len() - 1];
            let dt = last.2 - first.2;
            if dt > 0.0 {
                vx = ((last.0 - first.0) / dt).clamp(-max_v, max_v);
                vy = ((last.1 - first.1) / dt).clamp(-max_v, max_v);
            }
        }
        self.samples.clear();
        DragOutcome { vx, vy }
    }

    #[must_use]
    pub fn smooth_speed(&self) -> f64 {
        self.smooth_dx.hypot(self.smooth_dy)
    }
}
