use super::state::DesklingPhysics;
use crate::config::PhysicsConfig;

pub fn integrate(deskling: &mut DesklingPhysics, dt: f64, phys: &PhysicsConfig) -> bool {
    deskling.vy += phys.gravity * dt;
    deskling.y += deskling.vy * dt;
    deskling.x += deskling.vx * dt;

    let bounds = deskling.active_monitor;
    let right_bound = bounds.max_x - deskling.size;
    let floor = bounds.max_y - deskling.size;

    if deskling.y < bounds.min_y {
        deskling.y = bounds.min_y;
        deskling.vy = deskling.vy.abs() * phys.bounce_damping;
    }

    if deskling.x < bounds.min_x {
        deskling.x = bounds.min_x;
        deskling.vx = deskling.vx.abs() * phys.bounce_damping;
    } else if deskling.x > right_bound {
        deskling.x = right_bound;
        deskling.vx = -deskling.vx.abs() * phys.bounce_damping;
    }

    let mut floor_bounced = false;
    if deskling.y >= floor {
        deskling.y = floor;
        if deskling.vy.abs() < phys.min_velocity {
            deskling.vy = 0.0;
            deskling.vx = 0.0;
        } else {
            floor_bounced = true;
            deskling.vy = -deskling.vy * phys.bounce_damping;
            deskling.vx *= 0.8;
        }
    }

    deskling.vx *= phys.air_drag;
    floor_bounced
}
