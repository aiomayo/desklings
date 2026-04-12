#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Rect {
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_origin_size(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self::new(x, y, x + w, y + h)
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn clamp_point(&self, x: f64, y: f64) -> (f64, f64) {
        (x.clamp(self.min_x, self.max_x), y.clamp(self.min_y, self.max_y))
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    pub fn distance_sq_to(&self, x: f64, y: f64) -> f64 {
        let dx = if x < self.min_x {
            self.min_x - x
        } else if x > self.max_x {
            x - self.max_x
        } else {
            0.0
        };
        let dy = if y < self.min_y {
            self.min_y - y
        } else if y > self.max_y {
            y - self.max_y
        } else {
            0.0
        };
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone)]
pub struct MonitorLayout {
    pub monitors: Vec<Rect>,
    pub virtual_bounds: Rect,
}

impl MonitorLayout {
    pub fn from_rects(monitors: Vec<Rect>) -> Self {
        let virtual_bounds = monitors
            .iter()
            .copied()
            .reduce(Rect::union)
            .unwrap_or(Rect::new(0.0, 0.0, 1920.0, 1080.0));
        Self {
            monitors,
            virtual_bounds,
        }
    }

    pub fn from_window(window: &tauri::WebviewWindow) -> Self {
        let monitors: Vec<Rect> = window
            .available_monitors()
            .ok()
            .unwrap_or_default()
            .into_iter()
            .map(|m| {
                let scale = m.scale_factor();
                let pos = m.position();
                let size = m.size();
                Rect::from_origin_size(
                    f64::from(pos.x) / scale,
                    f64::from(pos.y) / scale,
                    f64::from(size.width) / scale,
                    f64::from(size.height) / scale,
                )
            })
            .collect();

        if monitors.is_empty() {
            return Self {
                monitors: vec![Rect::new(0.0, 0.0, 1920.0, 1080.0)],
                virtual_bounds: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            };
        }

        Self::from_rects(monitors)
    }

    pub fn monitor_at(&self, x: f64, y: f64) -> Rect {
        debug_assert!(!self.monitors.is_empty());

        if let Some(m) = self.monitors.iter().find(|m| m.contains_point(x, y)) {
            return *m;
        }

        self.monitors
            .iter()
            .copied()
            .min_by(|a, b| {
                a.distance_sq_to(x, y)
                    .partial_cmp(&b.distance_sq_to(x, y))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(self.virtual_bounds)
    }
}

