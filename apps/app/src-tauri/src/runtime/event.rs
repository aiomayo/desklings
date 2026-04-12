use serde::Serialize;

use crate::deskling::state::DesklingView;

pub const STATE_EVENT: &str = "deskling_state";

#[derive(Debug, Clone, Serialize)]
pub struct DesklingInfo {
    pub name: String,
    pub size: u32,
    pub sprites_dir: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesklingStateEvent<'a> {
    pub sprite: &'a str,
    pub flip: bool,
    pub mode: &'a str,
}

impl<'a> From<&'a DesklingView> for DesklingStateEvent<'a> {
    fn from(view: &'a DesklingView) -> Self {
        Self {
            sprite: view.sprite.as_str(),
            flip: view.flip,
            mode: view.mode.as_ref(),
        }
    }
}
