use std::io;
use std::path::PathBuf;

use serde::{Serialize, Serializer};

pub type DesklingResult<T> = Result<T, DesklingError>;

#[derive(Debug, thiserror::Error)]
pub enum DesklingError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse TOML at {path}: {source}")]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("deskling config invalid: {0}")]
    Compile(#[from] CompileError),

    #[error("deskling {slug:?} not found: {reason}")]
    DesklingNotFound { slug: String, reason: String },

    #[error("settings error: {0}")]
    Settings(String),

    #[error("missing window {0:?} (declared in tauri.conf.json)")]
    MissingWindow(&'static str),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("autostart error: {0}")]
    Autostart(String),
}

impl DesklingError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn toml_parse(path: impl Into<PathBuf>, source: toml::de::Error) -> Self {
        Self::TomlParse {
            path: path.into(),
            source,
        }
    }

    pub fn deskling_not_found(slug: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DesklingNotFound {
            slug: slug.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("animation {anim:?}: unknown mode {mode:?} (expected idle/falling/bounced/dragging)")]
    UnknownMode { anim: String, mode: String },

    #[error(
        "animation {anim:?}: event must set exactly one of `on` or `while` \
         (got neither or both)"
    )]
    EventVerbInvalid { anim: String },

    #[error(
        "animation {anim:?}: `{field}` is only valid on `on` events"
    )]
    EventFieldWrongVerb { anim: String, field: &'static str },

    #[error("animation {anim:?}: condition parse error: {msg}")]
    BadCondition { anim: String, msg: String },

    #[error(
        "no animation subscribes to `on = \"idle\"`; the deskling needs at \
         least one idle animation"
    )]
    NoIdleEvent,

    #[error(
        "no animation has a matching `while = \"{mode}\"` event at speed 0; \
         the deskling cannot resolve an animation for this mode"
    )]
    ModeUncovered { mode: &'static str },
}

impl Serialize for DesklingError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format_chain(self))
    }
}

pub fn format_chain(err: &(dyn std::error::Error + 'static)) -> String {
    let mut out = err.to_string();
    let mut src = err.source();
    while let Some(s) = src {
        out.push_str("\n  caused by: ");
        out.push_str(&s.to_string());
        src = s.source();
    }
    out
}


