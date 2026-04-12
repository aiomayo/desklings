use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DesklingConfig {
    pub deskling: DesklingMeta,
    pub physics: PhysicsConfig,
    pub drag: DragConfig,
    pub animations: BTreeMap<String, AnimationDef>,
}

#[derive(Debug, Deserialize)]
pub struct DesklingMeta {
    pub name: String,
    pub size: u32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct PhysicsConfig {
    pub gravity: f64,
    pub bounce_damping: f64,
    pub air_drag: f64,
    pub min_velocity: f64,
    pub max_throw_velocity: f64,
}

#[derive(Debug, Deserialize)]
pub struct DragConfig {
    pub smoothing_alpha: f64,
    pub direction_hysteresis: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnimationDef {
    pub frames: Vec<FrameDef>,
    #[serde(default = "default_true")]
    pub loops: bool,
    #[serde(default)]
    pub speed: f64,
    #[serde(default)]
    pub events: Vec<EventDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FrameDef {
    pub sprite: String,
    #[serde(default = "default_frame_duration")]
    pub duration: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventDef {
    #[serde(default)]
    pub on: Option<String>,
    #[serde(default, rename = "while")]
    pub while_: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub chance: Option<u32>,
    #[serde(default)]
    pub duration: Option<[f64; 2]>,
}

const fn default_true() -> bool {
    true
}

const fn default_frame_duration() -> f64 {
    1.0
}
