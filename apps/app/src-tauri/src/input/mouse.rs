use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use mouce::common::{MouseButton, MouseEvent};
use mouce::{Mouse as MouceManager, MouseActions};
use tracing::{error, info};

use crate::util::lock;

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseButtonState {
    pub left_down: bool,
    pub release_count: u64,
}

#[derive(Debug, Default)]
struct Shutdown {
    flag: Mutex<bool>,
    cv: Condvar,
}

impl Shutdown {
    fn signal(&self) {
        *lock(&self.flag) = true;
        self.cv.notify_all();
    }

    fn wait(&self) {
        let guard = lock(&self.flag);
        let _unused = self
            .cv
            .wait_while(guard, |set| !*set)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

#[derive(Debug)]
struct MouseHookInner {
    left_down: AtomicBool,
    release_count: AtomicU64,
    shutdown: Shutdown,
}

#[derive(Debug)]
pub struct MouseHook {
    inner: Arc<MouseHookInner>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl MouseHook {
    #[must_use]
    pub fn spawn() -> Arc<Self> {
        let inner = Arc::new(MouseHookInner {
            left_down: AtomicBool::new(false),
            release_count: AtomicU64::new(0),
            shutdown: Shutdown::default(),
        });

        let hook_inner = Arc::clone(&inner);
        let join = std::thread::Builder::new()
            .name("desklings-mouse-hook".into())
            .spawn(move || run_hook(&hook_inner))
            .map_err(|e| error!(error = %e, "failed to spawn mouse hook thread"))
            .ok();

        Arc::new(Self {
            inner,
            join: Mutex::new(join),
        })
    }

    pub fn snapshot(&self) -> MouseButtonState {
        MouseButtonState {
            left_down: self.inner.left_down.load(Ordering::Acquire),
            release_count: self.inner.release_count.load(Ordering::Acquire),
        }
    }
}

impl Drop for MouseHook {
    fn drop(&mut self) {
        self.inner.shutdown.signal();
        if let Some(handle) = lock(&self.join).take() {
            if let Err(e) = handle.join() {
                error!("mouse hook thread panicked on shutdown: {e:?}");
            }
        }
    }
}

fn run_hook(inner: &Arc<MouseHookInner>) {
    let mut manager = MouceManager::new();
    let cb_inner = Arc::clone(inner);
    let hook_result = manager.hook(Box::new(move |event| {
        if matches!(event, MouseEvent::Press(MouseButton::Left)) {
            cb_inner.left_down.store(true, Ordering::Release);
        } else if matches!(event, MouseEvent::Release(MouseButton::Left)) {
            cb_inner.left_down.store(false, Ordering::Release);
            cb_inner.release_count.fetch_add(1, Ordering::AcqRel);
        }
    }));

    if let Err(e) = hook_result {
        error!(error = ?e, "failed to install global mouse hook");
        return;
    }

    info!("mouse hook installed");

    inner.shutdown.wait();
    drop(manager);
}

#[must_use]
pub fn cursor_position() -> Option<(f64, f64)> {
    use mouse_position::mouse_position::Mouse;
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Some((f64::from(x), f64::from(y))),
        Mouse::Error => None,
    }
}
