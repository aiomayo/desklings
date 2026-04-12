use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::{DesklingError, DesklingResult};
use crate::util::lock;

const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppSettings {
    #[serde(deserialize_with = "deserialize_active_desklings")]
    pub active_desklings: BTreeMap<String, u32>,

    #[serde(default, skip_serializing)]
    pub active_deskling: String,

    pub locale: Option<String>,

    pub theme: Theme,
}

fn deserialize_active_desklings<'de, D>(d: D) -> Result<BTreeMap<String, u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Form {
        Map(BTreeMap<String, u32>),
        List(Vec<String>),
    }

    Ok(match Form::deserialize(d)? {
        Form::Map(m) => m.into_iter().filter(|(_, q)| *q > 0).collect(),
        Form::List(list) => list.into_iter().map(|s| (s, 1u32)).collect(),
    })
}

impl AppSettings {
    fn migrate(&mut self) {
        if !self.active_deskling.is_empty() {
            self.active_desklings
                .entry(std::mem::take(&mut self.active_deskling))
                .or_insert(1);
        }
        self.active_deskling.clear();
    }
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    inner: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> DesklingResult<Self> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| DesklingError::Settings(format!("resolve config dir: {e}")))?;
        let path = dir.join(SETTINGS_FILE);
        Self::load_from_path(path)
    }

    pub fn load_from_path(path: PathBuf) -> DesklingResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| DesklingError::io(parent, source))?;
        }

        let mut settings = match fs::read_to_string(&path) {
            Ok(body) => toml::from_str::<AppSettings>(&body)
                .map_err(|source| DesklingError::toml_parse(&path, source))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let initial = AppSettings::default();
                write_atomic(&path, &initial)?;
                initial
            }
            Err(source) => return Err(DesklingError::io(&path, source)),
        };
        settings.migrate();

        Ok(Self {
            path,
            inner: Mutex::new(settings),
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> AppSettings {
        lock(&self.inner).clone()
    }

    pub fn set_deskling_quantity(&self, slug: &str, quantity: u32) -> DesklingResult<()> {
        self.mutate(|s| {
            if quantity == 0 {
                s.active_desklings.remove(slug);
            } else {
                s.active_desklings.insert(slug.to_string(), quantity);
            }
        })
    }

    pub fn disable_deskling(&self, slug: &str) -> DesklingResult<()> {
        self.mutate(|s| {
            s.active_desklings.remove(slug);
        })
    }

    pub fn set_locale(&self, locale: Option<String>) -> DesklingResult<()> {
        self.mutate(|s| s.locale = locale)
    }

    pub fn set_theme(&self, theme: Theme) -> DesklingResult<()> {
        self.mutate(|s| s.theme = theme)
    }

    fn mutate<F: FnOnce(&mut AppSettings)>(&self, f: F) -> DesklingResult<()> {
        let mut guard = lock(&self.inner);
        f(&mut guard);
        write_atomic(&self.path, &guard)
    }
}

fn write_atomic(path: &Path, settings: &AppSettings) -> DesklingResult<()> {
    let body = toml::to_string_pretty(settings)
        .map_err(|e| DesklingError::Settings(format!("serialize settings: {e}")))?;
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, body).map_err(|source| DesklingError::io(&tmp, source))?;
    fs::rename(&tmp, path).map_err(|source| DesklingError::io(path, source))?;
    Ok(())
}
