use std::borrow::Cow;

use super::monitors::{MonitorLayout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

impl Facing {
    pub const fn sign(self) -> f64 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DesklingPhysics {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub size: f64,
    pub facing: Facing,
    pub virtual_bounds: Rect,
    pub active_monitor: Rect,
}

impl DesklingPhysics {
    pub fn new(layout: &MonitorLayout, size: f64) -> Self {
        let primary = layout
            .monitors
            .first()
            .copied()
            .unwrap_or(layout.virtual_bounds);

        Self {
            x: primary.min_x + primary.width() / 2.0 - size / 2.0,
            y: primary.min_y + 100.0,
            vx: 0.0,
            vy: 0.0,
            size,
            facing: Facing::Left,
            virtual_bounds: layout.virtual_bounds,
            active_monitor: primary,
        }
    }

    pub fn apply_layout(&mut self, layout: &MonitorLayout) {
        self.virtual_bounds = layout.virtual_bounds;
        let (cx, cy) = self.centre();
        self.active_monitor = layout.monitor_at(cx, cy);

        let (clamped_x, clamped_y) = self.virtual_bounds.clamp_point(self.x, self.y);
        self.x = clamped_x.min(self.virtual_bounds.max_x - self.size);
        self.y = clamped_y.min(self.virtual_bounds.max_y - self.size);
    }

    pub fn rebind_active_monitor_if_needed(&mut self, layout: &MonitorLayout) {
        let (cx, cy) = self.centre();
        if !self.active_monitor.contains_point(cx, cy) {
            self.active_monitor = layout.monitor_at(cx, cy);
        }
    }

    pub fn centre(&self) -> (f64, f64) {
        (self.x + self.size / 2.0, self.y + self.size / 2.0)
    }

    pub fn floor_y(&self) -> f64 {
        self.active_monitor.max_y - self.size
    }

    pub fn on_floor(&self) -> bool {
        (self.y - self.floor_y()).abs() < 1.0
    }
}

#[derive(Debug, Clone)]
pub struct DesklingView {
    pub x: f64,
    pub y: f64,
    pub sprite: String,
    pub flip: bool,
    pub mode: Cow<'static, str>,
}
