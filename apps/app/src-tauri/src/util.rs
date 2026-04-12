use std::sync::{Mutex, MutexGuard, PoisonError};

#[inline]
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

