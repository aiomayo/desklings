use crate::error::CompileError;

use super::compiled::{
    AnimationId, CompiledAnimation, CompiledConfig, CompiledDesklingMeta, CompiledDrag,
    CompiledEvent, EventKind, Mode,
};
use super::condition::{self, Cond, Context};
use super::schema::{AnimationDef, DesklingConfig, EventDef};

pub fn compile(raw: DesklingConfig) -> Result<CompiledConfig, CompileError> {
    let animations = raw
        .animations
        .into_iter()
        .enumerate()
        .map(|(i, (name, def))| compile_animation(AnimationId(i), name, def))
        .collect::<Result<Vec<_>, _>>()?;

    let cfg = CompiledConfig {
        deskling: CompiledDesklingMeta {
            name: raw.deskling.name,
            size: raw.deskling.size,
        },
        physics: raw.physics,
        drag: CompiledDrag {
            smoothing_alpha: raw.drag.smoothing_alpha,
            direction_hysteresis: raw.drag.direction_hysteresis,
        },
        animations,
    };

    validate_coverage(&cfg)?;

    Ok(cfg)
}

fn compile_animation(
    id: AnimationId,
    name: String,
    def: AnimationDef,
) -> Result<CompiledAnimation, CompileError> {
    let events = def
        .events
        .into_iter()
        .map(|e| compile_event(&name, e))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CompiledAnimation {
        id,
        name,
        frames: def.frames,
        loops: def.loops,
        speed: def.speed,
        events,
    })
}

fn compile_event(anim_name: &str, raw: EventDef) -> Result<CompiledEvent, CompileError> {
    let (mode_name, kind) = match (raw.on.as_deref(), raw.while_.as_deref()) {
        (Some(on), None) => {
            let kind = EventKind::Edge {
                chance: raw.chance.unwrap_or(1),
                duration: raw.duration,
            };
            (on.to_string(), kind)
        }
        (None, Some(w)) => {
            if raw.chance.is_some() {
                return Err(CompileError::EventFieldWrongVerb {
                    anim: anim_name.to_string(),
                    field: "chance",
                });
            }
            if raw.duration.is_some() {
                return Err(CompileError::EventFieldWrongVerb {
                    anim: anim_name.to_string(),
                    field: "duration",
                });
            }
            (w.to_string(), EventKind::Level)
        }
        _ => {
            return Err(CompileError::EventVerbInvalid {
                anim: anim_name.to_string(),
            });
        }
    };

    let mode = Mode::from_str(&mode_name).ok_or_else(|| CompileError::UnknownMode {
        anim: anim_name.to_string(),
        mode: mode_name,
    })?;

    let cond = match raw.condition.as_deref() {
        Some(src) => condition::parse(src).map_err(|e| CompileError::BadCondition {
            anim: anim_name.to_string(),
            msg: e.msg,
        })?,
        None => Cond::Always,
    };

    Ok(CompiledEvent { mode, cond, kind })
}

fn validate_coverage(cfg: &CompiledConfig) -> Result<(), CompileError> {
    let has_idle_edge = cfg.animations.iter().any(|a| {
        a.events
            .iter()
            .any(|e| e.mode == Mode::Idle && matches!(e.kind, EventKind::Edge { .. }))
    });
    if !has_idle_edge {
        return Err(CompileError::NoIdleEvent);
    }

    let probe_speeds = [0.0_f64, 400.0, 1e6];
    for mode in [Mode::Falling, Mode::Bounced, Mode::Dragging] {
        let covers_all = probe_speeds.iter().all(|&s| {
            cfg.animations.iter().any(|a| {
                a.events.iter().any(|e| {
                    e.mode == mode
                        && matches!(e.kind, EventKind::Level)
                        && e.cond.eval(&Context { speed: s })
                })
            })
        });
        if !covers_all {
            return Err(CompileError::ModeUncovered {
                mode: mode.as_str(),
            });
        }
    }
    Ok(())
}
