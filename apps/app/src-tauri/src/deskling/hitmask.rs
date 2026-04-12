use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use tracing::warn;

pub const ALPHA_THRESHOLD: u8 = 64;

#[derive(Debug)]
pub struct HitMask {
    width: u32,
    height: u32,
    alpha: Vec<u8>,
}

impl HitMask {
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        let decoder = png::Decoder::new(BufReader::new(file));
        let mut reader = decoder
            .read_info()
            .map_err(|e| format!("png header {}: {e}", path.display()))?;

        let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| format!("png frame {}: {e}", path.display()))?;

        if info.bit_depth != png::BitDepth::Eight {
            return Err(format!(
                "{}: only 8-bit PNGs are supported (got {:?})",
                path.display(),
                info.bit_depth
            ));
        }

        let width = info.width;
        let height = info.height;
        let pixels = (width as usize)
            .checked_mul(height as usize)
            .ok_or_else(|| format!("{}: dimensions overflow", path.display()))?;

        let alpha = match info.color_type {
            png::ColorType::Rgba => {
                let mut out = Vec::with_capacity(pixels);
                for chunk in buf.chunks_exact(4) {
                    out.push(chunk[3]);
                }
                out
            }
            png::ColorType::GrayscaleAlpha => {
                let mut out = Vec::with_capacity(pixels);
                for chunk in buf.chunks_exact(2) {
                    out.push(chunk[1]);
                }
                out
            }
            png::ColorType::Rgb | png::ColorType::Grayscale => vec![255u8; pixels],
            other => {
                return Err(format!(
                    "{}: unsupported color type {other:?}",
                    path.display()
                ));
            }
        };

        if alpha.len() != pixels {
            return Err(format!(
                "{}: decoded alpha length {} != {}x{}={}",
                path.display(),
                alpha.len(),
                width,
                height,
                pixels
            ));
        }

        Ok(Self {
            width,
            height,
            alpha,
        })
    }

    #[must_use]
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[must_use]
    pub fn is_opaque_at(&self, px: i32, py: i32, flipped: bool) -> bool {
        if px < 0 || py < 0 {
            return false;
        }
        let w = self.width as i32;
        let h = self.height as i32;
        if px >= w || py >= h {
            return false;
        }
        let src_x = if flipped { w - 1 - px } else { px };
        let idx = (py as usize) * self.width as usize + src_x as usize;
        self.alpha.get(idx).is_some_and(|&a| a > ALPHA_THRESHOLD)
    }
}

#[derive(Debug, Default)]
pub struct HitMaskCache {
    sprites_dir: PathBuf,
    masks: HashMap<String, Option<HitMask>>,
}

impl HitMaskCache {
    #[must_use]
    pub fn new(sprites_dir: PathBuf) -> Self {
        Self {
            sprites_dir,
            masks: HashMap::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.masks.clear();
    }

    pub fn get(&mut self, filename: &str) -> Option<&HitMask> {
        if !self.masks.contains_key(filename) {
            let full_path = self.sprites_dir.join(filename);
            let loaded = HitMask::load(&full_path)
                .map_err(|e| warn!(sprite = %filename, error = %e, "failed to load hitmask"))
                .ok();
            self.masks.insert(filename.to_string(), loaded);
        }
        self.masks.get(filename).and_then(Option::as_ref)
    }
}

#[must_use]
pub fn hit_test(
    mask: Option<&HitMask>,
    deskling_x: f64,
    deskling_y: f64,
    window_size: f64,
    cursor_x: f64,
    cursor_y: f64,
    flipped: bool,
) -> bool {
    if cursor_x < deskling_x
        || cursor_x > deskling_x + window_size
        || cursor_y < deskling_y
        || cursor_y > deskling_y + window_size
    {
        return false;
    }

    let Some(mask) = mask else {
        return true;
    };

    let lx = cursor_x - deskling_x;
    let ly = cursor_y - deskling_y;

    let (sprite_w, sprite_h) = mask.dimensions();
    let px = ((lx / window_size) * f64::from(sprite_w)).floor() as i32;
    let py = ((ly / window_size) * f64::from(sprite_h)).floor() as i32;

    mask.is_opaque_at(px, py, flipped)
}

