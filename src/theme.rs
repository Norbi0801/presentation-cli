use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct RawTheme {
    #[serde(default)]
    name: Option<String>,
    accent: String,
    dim: String,
    glow: String,
}

#[derive(Debug, Clone)]
pub struct ThemeSpec {
    label: String,
    palette: ThemePalette,
}

impl ThemeSpec {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn palette(&self) -> &ThemePalette {
        &self.palette
    }
}

#[derive(Debug, Clone)]
pub struct ThemePalette {
    accent: String,
    dim: String,
    glow: String,
}

impl ThemePalette {
    pub fn new(accent: impl Into<String>, dim: impl Into<String>, glow: impl Into<String>) -> Self {
        Self {
            accent: accent.into(),
            dim: dim.into(),
            glow: glow.into(),
        }
    }

    pub fn accent(&self) -> &str {
        &self.accent
    }

    pub fn dim(&self) -> &str {
        &self.dim
    }

    pub fn glow(&self) -> &str {
        &self.glow
    }
}

pub fn load_from_path(path: &Path) -> Result<ThemeSpec, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let raw: RawTheme = toml::from_str(&contents)?;

    let label = raw
        .name
        .or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .ok_or_else(|| format!("Plik motywu ({}) nie zawiera nazwy motywu", path.display()))?;

    Ok(ThemeSpec {
        label,
        palette: ThemePalette::new(raw.accent, raw.dim, raw.glow),
    })
}
