use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use mouce::common::{MouseButton, MouseEvent};
use mouce::{Mouse as MouceManager, MouseActions};
use tracing::{error, info, warn};

use crate::util::lock;

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseButtonState {
    pub release_count: u64,
}

#[cfg(target_os = "macos")]
pub fn is_accessibility_trusted() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(target_os = "macos")]
pub fn prompt_accessibility_trust() -> bool {
    use std::ffi::c_void;
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const u8,
            encoding: u32,
        ) -> *const c_void;
        fn CFDictionaryCreate(
            alloc: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            count: isize,
            key_cbs: *const c_void,
            val_cbs: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
        static kCFBooleanTrue: *const c_void;
    }
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    unsafe {
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            b"AXTrustedCheckOptionPrompt\0".as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );
        let keys = [key];
        let values = [kCFBooleanTrue];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            std::ptr::addr_of!(kCFTypeDictionaryKeyCallBacks),
            std::ptr::addr_of!(kCFTypeDictionaryValueCallBacks),
        );
        let result = AXIsProcessTrustedWithOptions(dict);
        CFRelease(dict);
        CFRelease(key);
        result
    }
}

#[cfg(target_os = "macos")]
fn poll_left_mouse_button() -> bool {
    use objc2_core_graphics::{CGEventSource, CGEventSourceStateID, CGMouseButton};
    CGEventSource::button_state(CGEventSourceStateID::CombinedSessionState, CGMouseButton::Left)
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
    hook_active: AtomicBool,
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
            hook_active: AtomicBool::new(false),
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
            release_count: self.inner.release_count.load(Ordering::Acquire),
        }
    }

    pub fn is_active(&self) -> bool {
        self.inner.hook_active.load(Ordering::Acquire)
    }

    pub fn left_down(&self) -> bool {
        if self.is_active() {
            return self.inner.left_down.load(Ordering::Acquire);
        }
        #[cfg(target_os = "macos")]
        {
            poll_left_mouse_button()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.inner.left_down.load(Ordering::Acquire)
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
    #[cfg(target_os = "macos")]
    if !is_accessibility_trusted() {
        warn!("accessibility permission not granted; global mouse hook skipped");
        inner.hook_active.store(false, Ordering::Release);
        inner.shutdown.wait();
        return;
    }

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
        inner.hook_active.store(false, Ordering::Release);
        return;
    }

    inner.hook_active.store(true, Ordering::Release);
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
