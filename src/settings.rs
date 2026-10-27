use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use windows::core::w;
use windows::Win32::System::Registry::{
    RegDeleteKeyValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
};

use crate::i18n::Lang;

const RUN_KEY: windows::core::PCWSTR =
    w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const RUN_VALUE: windows::core::PCWSTR = w!("PortZap");
/// Value name của bản cũ (KillPort) — dọn để không còn entry Run thừa trong registry.
const LEGACY_RUN_VALUE: windows::core::PCWSTR = w!("KillPort");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub lang: LangCode,
    pub autostart: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LangCode {
    En,
    Vi,
}

impl From<LangCode> for Lang {
    fn from(c: LangCode) -> Lang {
        match c {
            LangCode::En => Lang::En,
            LangCode::Vi => Lang::Vi,
        }
    }
}

impl From<Lang> for LangCode {
    fn from(l: Lang) -> LangCode {
        match l {
            Lang::En => LangCode::En,
            Lang::Vi => LangCode::Vi,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: LangCode::En,
            autostart: false,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata).join("portzap").join("settings.json"))
}

/// Path của bản cũ KillPort — chỉ đọc lại khi chưa có file mới (giữ cài đặt sau đổi tên).
fn legacy_settings_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata).join("killport").join("settings.json"))
}

impl Settings {
    pub fn load() -> Settings {
        let Some(path) = settings_path() else {
            return Settings::default();
        };
        fs::read_to_string(&path)
            .ok()
            .or_else(|| legacy_settings_path().and_then(|old| fs::read_to_string(old).ok()))
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = settings_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang.into()
    }
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    unsafe {
        let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, LEGACY_RUN_VALUE);
        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let cmd: Vec<u16> = format!("\"{}\"", exe.display())
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let rc = RegSetKeyValueW(
                HKEY_CURRENT_USER,
                RUN_KEY,
                RUN_VALUE,
                REG_SZ.0,
                Some(cmd.as_ptr().cast()),
                (cmd.len() * 2) as u32,
            );
            if rc.is_ok() {
                Ok(())
            } else {
                Err(format!("RegSetKeyValueW failed: {rc:?}"))
            }
        } else {
            let rc = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VALUE);
            if rc.is_ok() || rc.0 == 2 {
                Ok(())
            } else {
                Err(format!("RegDeleteKeyValueW failed: {rc:?}"))
            }
        }
    }
}
