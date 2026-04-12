use super::condition::Cond;
use super::schema::{FrameDef, PhysicsConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Idle,
    Falling,
    Bounced,
    Dragging,
}

impl Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Falling => "falling",
            Self::Bounced => "bounced",
            Self::Dragging => "dragging",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "idle" => Self::Idle,
            "falling" => Self::Falling,
            "bounced" => Self::Bounced,
            "dragging" => Self::Dragging,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Edge {
        chance: u32,
        duration: Option<[f64; 2]>,
    },
    Level,
}

#[derive(Debug, Clone)]
pub struct CompiledEvent {
    pub mode: Mode,
    pub cond: Cond,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub struct CompiledAnimation {
    pub id: AnimationId,
    #[allow(dead_code)]
    pub name: String,
    pub frames: Vec<FrameDef>,
    pub loops: bool,
    pub speed: f64,
    pub events: Vec<CompiledEvent>,
}

#[derive(Debug, Clone)]
pub struct CompiledDrag {
    pub smoothing_alpha: f64,
    pub direction_hysteresis: f64,
}

#[derive(Debug, Clone)]
pub struct CompiledDesklingMeta {
    pub name: String,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct CompiledConfig {
    pub deskling: CompiledDesklingMeta,
    pub physics: PhysicsConfig,
    pub drag: CompiledDrag,
    pub animations: Vec<CompiledAnimation>,
}

