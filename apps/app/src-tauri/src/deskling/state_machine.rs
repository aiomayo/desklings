use fastrand::Rng;

use crate::config::{AnimationId, CompiledConfig, Context, EventKind, Mode};

use super::animator::{AnimationPlayer, AnimationTick};
use super::falling;
use super::mover;
use super::state::{DesklingPhysics, DesklingView, Facing};

#[derive(Debug)]
pub struct StateMachine {
    mode: Mode,
    animator: AnimationPlayer,
    elapsed: f64,
    duration: f64,
    has_bounced: bool,
}

impl StateMachine {
    pub fn new(_cfg: &CompiledConfig) -> Self {
        let seed = AnimationId(0);
        Self {
            mode: Mode::Falling,
            animator: AnimationPlayer::new(seed),
            elapsed: 0.0,
            duration: f64::INFINITY,
            has_bounced: false,
        }
    }

    pub fn reset_for(&mut self, cfg: &CompiledConfig) {
        *self = Self::new(cfg);
    }

    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub fn enter_falling(&mut self, _cfg: &CompiledConfig) {
        self.mode = Mode::Falling;
        self.has_bounced = false;
        self.elapsed = 0.0;
        self.duration = f64::INFINITY;
    }

    pub fn enter_dragging(&mut self, _cfg: &CompiledConfig) {
        self.mode = Mode::Dragging;
        self.elapsed = 0.0;
        self.duration = f64::INFINITY;
    }

    pub fn tick(
        &mut self,
        deskling: &mut DesklingPhysics,
        cfg: &CompiledConfig,
        rng: &mut Rng,
        ctx: &Context,
        dt: f64,
    ) -> DesklingView {
        match self.mode {
            Mode::Falling | Mode::Bounced => self.tick_airborne(deskling, cfg, rng, ctx, dt),
            Mode::Idle => self.tick_idle(deskling, cfg, rng, ctx, dt),
            Mode::Dragging => self.tick_dragging(cfg, ctx, dt),
        }
        self.build_view(deskling, cfg)
    }

    fn tick_airborne(
        &mut self,
        deskling: &mut DesklingPhysics,
        cfg: &CompiledConfig,
        rng: &mut Rng,
        ctx: &Context,
        dt: f64,
    ) {
        let target = pick_while(cfg, self.mode, ctx)
            .unwrap_or_else(|| self.animator.id());
        if self.animator.id() != target {
            self.animator = AnimationPlayer::new(target);
        }
        self.animator.tick(dt, cfg);

        let bounced = falling::integrate(deskling, dt, &cfg.physics);
        if bounced && !self.has_bounced {
            self.has_bounced = true;
            self.mode = Mode::Bounced;
        }

        const FACING_DEADZONE: f64 = 1.0;
        if deskling.vx > FACING_DEADZONE {
            deskling.facing = Facing::Right;
        } else if deskling.vx < -FACING_DEADZONE {
            deskling.facing = Facing::Left;
        }

        if deskling.on_floor() && deskling.vy.abs() < cfg.physics.min_velocity {
            self.enter_idle(cfg, rng, ctx);
        }
    }

    fn tick_idle(
        &mut self,
        deskling: &mut DesklingPhysics,
        cfg: &CompiledConfig,
        rng: &mut Rng,
        ctx: &Context,
        dt: f64,
    ) {
        self.elapsed += dt;

        let anim_result = self.animator.tick(dt, cfg);
        let speed = cfg.animations[self.animator.id().0].speed;
        mover::apply(deskling, dt, speed);

        let frames_ended = anim_result == AnimationTick::Finished;
        let duration_expired = self.elapsed >= self.duration;
        if frames_ended || duration_expired {
            self.pick_idle(cfg, rng, ctx);
        }
    }

    fn tick_dragging(&mut self, cfg: &CompiledConfig, ctx: &Context, dt: f64) {
        let target = pick_while(cfg, Mode::Dragging, ctx)
            .unwrap_or_else(|| self.animator.id());
        if self.animator.id() != target {
            self.animator = AnimationPlayer::new(target);
        }
        self.animator.tick(dt, cfg);
    }

    pub fn enter_idle(&mut self, cfg: &CompiledConfig, rng: &mut Rng, ctx: &Context) {
        self.mode = Mode::Idle;
        self.has_bounced = false;
        self.pick_idle(cfg, rng, ctx);
    }

    fn pick_idle(&mut self, cfg: &CompiledConfig, rng: &mut Rng, ctx: &Context) {
        let mut candidates: Vec<(AnimationId, u32, Option<[f64; 2]>)> = Vec::new();
        let mut total: u32 = 0;
        for anim in &cfg.animations {
            for ev in &anim.events {
                if ev.mode != Mode::Idle {
                    continue;
                }
                let EventKind::Edge { chance, duration } = ev.kind else {
                    continue;
                };
                if !ev.cond.eval(ctx) {
                    continue;
                }
                total = total.saturating_add(chance);
                candidates.push((anim.id, chance, duration));
            }
        }

        let (picked_id, picked_duration) = if candidates.is_empty() || total == 0 {
            (AnimationId(0), None)
        } else {
            let mut roll = rng.u32(..total);
            let mut chosen = (candidates[0].0, candidates[0].2);
            for (id, chance, duration) in &candidates {
                if roll < *chance {
                    chosen = (*id, *duration);
                    break;
                }
                roll -= *chance;
            }
            chosen
        };

        self.animator = AnimationPlayer::new(picked_id);
        self.elapsed = 0.0;
        self.duration = match picked_duration {
            Some([a, b]) => rng.f64().mul_add(b - a, a),
            None => f64::INFINITY,
        };
    }

    fn build_view(&self, deskling: &DesklingPhysics, cfg: &CompiledConfig) -> DesklingView {
        use std::borrow::Cow;

        let sprite = self.animator.current_sprite(cfg).to_string();
        let mode: Cow<'static, str> = Cow::Borrowed(self.mode.as_str());

        DesklingView {
            x: deskling.x,
            y: deskling.y,
            sprite,
            flip: matches!(deskling.facing, Facing::Right),
            mode,
        }
    }
}

fn pick_while(cfg: &CompiledConfig, mode: Mode, ctx: &Context) -> Option<AnimationId> {
    for anim in &cfg.animations {
        for ev in &anim.events {
            if ev.mode != mode || !matches!(ev.kind, EventKind::Level) {
                continue;
            }
            if ev.cond.eval(ctx) {
                return Some(anim.id);
            }
        }
    }
    None
}
