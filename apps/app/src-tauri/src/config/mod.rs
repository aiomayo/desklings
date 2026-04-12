pub mod compile;
pub mod compiled;
pub mod condition;
pub mod live;
pub mod loader;
pub mod schema;
pub mod settings;

pub use compiled::{
    AnimationId, CompiledAnimation, CompiledConfig, CompiledDrag, EventKind, Mode,
};
pub use condition::Context;
pub use live::{spawn_watcher, DesklingReloadedEvent, LiveConfig, RELOAD_EVENT};
pub use loader::{
    first_installed_slug, list_installed_desklings, load_deskling_by_slug, user_desklings_dir,
    DesklingSummary,
};
pub use schema::PhysicsConfig;
pub use settings::{AppSettings, SettingsStore, Theme};
