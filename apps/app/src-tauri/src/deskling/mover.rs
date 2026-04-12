use super::state::{DesklingPhysics, Facing};

pub fn apply(deskling: &mut DesklingPhysics, dt: f64, speed: f64) {
    if speed == 0.0 {
        return;
    }

    deskling.x += deskling.facing.sign() * speed * dt;

    let left_bound = deskling.active_monitor.min_x;
    let right_bound = deskling.active_monitor.max_x - deskling.size;
    if deskling.x <= left_bound {
        deskling.x = left_bound;
        deskling.facing = Facing::Right;
    } else if deskling.x >= right_bound {
        deskling.x = right_bound;
        deskling.facing = Facing::Left;
    }
}
