use std::collections::HashMap;
use std::sync::Mutex;

use crate::util::lock;

#[derive(Debug, Default)]
pub struct CursorArbiter {
    hits: Mutex<HashMap<String, bool>>,
    z_order: Mutex<Vec<String>>,
}

impl CursorArbiter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish_hit(&self, instance_id: &str, hit: bool) {
        let mut hits = lock(&self.hits);
        hits.insert(instance_id.to_string(), hit);
    }

    pub fn should_claim(&self, instance_id: &str) -> bool {
        let hits = lock(&self.hits);
        let z_order = lock(&self.z_order);
        for candidate in z_order.iter().rev() {
            if hits.get(candidate).copied().unwrap_or(false) {
                return candidate == instance_id;
            }
        }
        false
    }

    pub fn push_top(&self, instance_id: &str) {
        let mut z_order = lock(&self.z_order);
        z_order.retain(|s| s != instance_id);
        z_order.push(instance_id.to_string());
    }

    pub fn remove(&self, instance_id: &str) {
        lock(&self.hits).remove(instance_id);
        lock(&self.z_order).retain(|s| s != instance_id);
    }
}
