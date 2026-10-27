use std::thread;

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::i18n::Lang;
use crate::{Bus, Event};

pub const ID_EXIT: &str = "exit";
pub const ID_LANG: &str = "lang";
pub const ID_AUTOSTART: &str = "autostart";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCmd {
    Exit,
    ToggleLang,
    ToggleAutostart,
}

pub struct TrayHandles {
    /// Phải giữ alive, nếu drop thì icon biến mất khỏi system tray.
    #[allow(dead_code)]
    pub icon: TrayIcon,
    pub lang_item: MenuItem,
    pub autostart_item: MenuItem,
    pub exit_item: MenuItem,
}

pub fn init(bus: Bus, lang: Lang, autostart: bool) -> TrayHandles {
    let menu = Menu::new();
    let lang_item = MenuItem::with_id(ID_LANG, lang.menu_lang_switch_hint(), true, None);
    let autostart_item =
        MenuItem::with_id(ID_AUTOSTART, lang.menu_autostart(autostart), true, None);
    let exit_item = MenuItem::with_id(ID_EXIT, lang.menu_exit(), true, None);
    let _ = menu.append(&lang_item);
    let _ = menu.append(&autostart_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&exit_item);

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("PortZap - Alt+K")
        .with_icon(make_icon())
        .build()
        .expect("failed to build tray icon");

    spawn_bridge(bus);

    TrayHandles {
        icon: tray,
        lang_item,
        autostart_item,
        exit_item,
    }
}

fn spawn_bridge(bus: Bus) {
    thread::spawn(move || {
        let rx = MenuEvent::receiver();
        while let Ok(ev) = rx.recv() {
            let cmd = match ev.id().as_ref() {
                ID_EXIT => TrayCmd::Exit,
                ID_LANG => TrayCmd::ToggleLang,
                ID_AUTOSTART => TrayCmd::ToggleAutostart,
                _ => continue,
            };
            bus.send(Event::Cmd(cmd));
        }
    });
}

/// Zap trong viewBox 24x24, tâm (12,12) — khớp với icons/icon.ico.
const BOLT: [(f32, f32); 6] = [
    (13.0, 2.0),
    (3.0, 14.0),
    (12.0, 14.0),
    (11.0, 22.0),
    (21.0, 10.0),
    (12.0, 10.0),
];

fn in_bolt(bx: f32, by: f32) -> bool {
    let mut inside = false;
    let mut prev = BOLT[BOLT.len() - 1];
    for &cur in &BOLT {
        let (xi, yi) = cur;
        let (xj, yj) = prev;
        if (yi > by) != (yj > by) && bx < (xj - xi) * (by - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        prev = cur;
    }
    inside
}

/// RGBA cho đĩa xanh lá + tia zap trắng (khớp góc nhìn với icons/icon.ico).
fn icon_rgba(s: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (s * s * 4) as usize];
    let c = (s as f32 - 1.0) / 2.0;
    let k = s as f32 / 24.0 * 0.96;
    let ring_w = 1.4f32;
    for y in 0..s {
        for x in 0..s {
            let dx = x as f32 - c;
            let dy = y as f32 - c;
            let d = (dx * dx + dy * dy).sqrt();
            let i = ((y * s + x) * 4) as usize;
            if d > c - 0.5 {
                continue;
            }
            let t = (y as f32 / (s - 1) as f32).clamp(0.0, 1.0);
            let (r, g, b) = if d >= c - 0.5 - ring_w {
                (6u8, 95u8, 70u8)
            } else {
                (
                    (34.0 + (21.0 - 34.0) * t) as u8,
                    (197.0 + (128.0 - 197.0) * t) as u8,
                    (94.0 + (61.0 - 94.0) * t) as u8,
                )
            };
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
            if in_bolt(12.0 + dx / k, 12.0 + dy / k) {
                rgba[i] = 255;
                rgba[i + 1] = 255;
                rgba[i + 2] = 255;
            }
        }
    }
    rgba
}

fn make_icon() -> Icon {
    let s = 32u32;
    Icon::from_rgba(icon_rgba(s), s, s).expect("valid icon")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_icon_shape() {
        let s = 32u32;
        let px = icon_rgba(s);
        let at = |x: u32, y: u32| {
            let i = ((y * s + x) * 4) as usize;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };

        assert_eq!(at(0, 0)[3], 0, "góc ngoài đĩa phải trong suốt");
        assert_eq!(at(s - 1, s - 1)[3], 0, "góc ngoài đĩa phải trong suốt");
        assert_eq!(at(16, 16), [255, 255, 255, 255], "tâm là tia zap trắng");
        assert_eq!(at(16, 1), [6, 95, 70, 255], "viền là emerald đậm");
        assert_eq!(at(4, 16)[3], 255, "thân đĩa liền mạch");
        assert_ne!(at(4, 16), [255, 255, 255, 255], "thân đĩa không nằm trong tia zap");
    }
}
