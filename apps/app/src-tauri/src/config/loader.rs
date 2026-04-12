use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{DesklingError, DesklingResult};

use super::compile::compile;
use super::compiled::CompiledConfig;
use super::schema::DesklingConfig;

const DESKLING_TOML: &str = "deskling.toml";
const DESKLINGS_SUBDIR: &str = "desklings";

#[derive(Debug)]
pub struct LoadedDeskling {
    pub config: CompiledConfig,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesklingSummary {
    pub slug: String,
    pub name: String,
    pub size: u32,
    pub preview: Option<String>,
}

const DESKLINGS_DIR_ENV: &str = "DESKLINGS_DIR";

pub fn user_desklings_dir(app: &AppHandle) -> DesklingResult<PathBuf> {
    if let Some(raw) = std::env::var_os(DESKLINGS_DIR_ENV) {
        let dir = PathBuf::from(raw);
        fs::create_dir_all(&dir).map_err(|source| DesklingError::io(&dir, source))?;
        tracing::info!(dir = %dir.display(), "DESKLINGS_DIR override active");
        return Ok(dir);
    }

    #[cfg(debug_assertions)]
    {
        let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join(DESKLINGS_SUBDIR));
        if let Some(dir) = repo_dir {
            fs::create_dir_all(&dir).map_err(|source| DesklingError::io(&dir, source))?;
            tracing::info!(
                dir = %dir.display(),
                "debug build: using repo-local desklings dir"
            );
            return Ok(dir);
        }
    }

    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| DesklingError::Settings(format!("resolve app data dir: {e}")))?;
    let dir = base.join(DESKLINGS_SUBDIR);
    fs::create_dir_all(&dir).map_err(|source| DesklingError::io(&dir, source))?;
    Ok(dir)
}

pub fn load_deskling_by_slug(desklings_dir: &Path, slug: &str) -> DesklingResult<LoadedDeskling> {
    let deskling_dir = desklings_dir.join(slug);
    let deskling_toml = deskling_dir.join(DESKLING_TOML);
    let sprites_dir = deskling_dir.join("sprites");

    if !deskling_toml.exists() {
        return Err(DesklingError::deskling_not_found(
            slug,
            format!("missing {}", deskling_toml.display()),
        ));
    }
    if !sprites_dir.is_dir() {
        return Err(DesklingError::deskling_not_found(
            slug,
            format!("missing sprites folder at {}", sprites_dir.display()),
        ));
    }

    let body = fs::read_to_string(&deskling_toml)
        .map_err(|source| DesklingError::io(&deskling_toml, source))?;
    let raw: DesklingConfig = toml::from_str(&body)
        .map_err(|source| DesklingError::toml_parse(&deskling_toml, source))?;
    let config = compile(raw)?;

    Ok(LoadedDeskling {
        config,
        slug: slug.to_string(),
    })
}

pub fn list_installed_desklings(desklings_dir: &Path) -> Vec<DesklingSummary> {
    #[derive(serde::Deserialize)]
    struct Header {
        deskling: HeaderMeta,
    }
    #[derive(serde::Deserialize)]
    struct HeaderMeta {
        name: String,
        size: u32,
    }

    let Ok(entries) = fs::read_dir(desklings_dir) else {
        return Vec::new();
    };

    let mut out: Vec<DesklingSummary> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(slug) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        let deskling_toml = path.join(DESKLING_TOML);
        let Ok(body) = fs::read_to_string(&deskling_toml) else {
            continue;
        };
        let Ok(header) = toml::from_str::<Header>(&body) else {
            continue;
        };

        let preview = find_preview_sprite(&path.join("sprites"));
        out.push(DesklingSummary {
            slug,
            name: header.deskling.name,
            size: header.deskling.size,
            preview,
        });
    }

    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn first_installed_slug(desklings_dir: &Path) -> Option<String> {
    list_installed_desklings(desklings_dir)
        .into_iter()
        .next()
        .map(|s| s.slug)
}

fn find_preview_sprite(sprites_dir: &Path) -> Option<String> {
    let mut pngs: Vec<PathBuf> = fs::read_dir(sprites_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("png"))
        })
        .collect();
    pngs.sort();

    let idle = pngs.iter().find(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("idle"))
    });

    idle.or_else(|| pngs.first())
        .map(|p| p.to_string_lossy().into_owned())
}
