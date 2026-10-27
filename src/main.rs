#![windows_subsystem = "windows"]

const UI_FINGERPRINT: &str = include_str!(concat!(env!("OUT_DIR"), "/ui_fingerprint.txt"));

mod commands;
mod hang;
mod hotkey;
mod i18n;
mod port;
mod proc;
mod settings;
mod toast;
mod tray;

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use commands::Command;
use i18n::Lang;
use proc::KillReport;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tray::TrayCmd;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    CreateEllipticRgn, DeleteObject, SetWindowRgn, HGDIOBJ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, SetFocus, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_LMENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetForegroundWindow, GetWindow, MessageBoxW, SetForegroundWindow,
    ShowWindow, SwitchToThisWindow, GW_CHILD, MB_ICONWARNING, MB_OK, SW_SHOW,
};

#[derive(Debug)]
pub enum Event {
    Hotkey,
    HotkeyFailed(String),
    Cmd(TrayCmd),
}

#[derive(Clone)]
pub struct Bus {
    tx: Sender<Event>,
}

impl Bus {
    pub fn send(&self, e: Event) {
        let _ = self.tx.send(e);
    }
}

thread_local! {
    static TRAY: RefCell<Option<tray::TrayHandles>> = const { RefCell::new(None) };
}

static LAST_OPEN: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn set_disc_region(hwnd: HWND, w: u32, h: u32) {
    if w == 0 || h == 0 {
        return;
    }
    unsafe {
        let rgn = CreateEllipticRgn(0, 0, w as i32, h as i32);
        if rgn.is_invalid() {
            return;
        }
        if SetWindowRgn(hwnd, Some(rgn), true) == 0 {
            let _ = DeleteObject(HGDIOBJ(rgn.0));
        }
    }
}

fn activate_overlay(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        if GetForegroundWindow() != hwnd {
            keybd_event(VK_LMENU.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_LMENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
            let _ = SetForegroundWindow(hwnd);
        }
        if GetForegroundWindow() != hwnd {
            SwitchToThisWindow(hwnd, true);
        }
        let _ = BringWindowToTop(hwnd);
        if let Ok(child) = GetWindow(hwnd, GW_CHILD) {
            let _ = SetFocus(Some(child));
        }
    }
}

fn arm_window_events(win: &WebviewWindow) {
    let Ok(hwnd) = win.hwnd() else {
        return;
    };
    let hv = hwnd.0 as isize;
    let w = win.clone();
    win.on_window_event(move |event| match event {
        tauri::WindowEvent::ScaleFactorChanged {
            new_inner_size, ..
        } => {
            set_disc_region(HWND(hv as _), new_inner_size.width, new_inner_size.height);
        }
        tauri::WindowEvent::Resized(size) => {
            set_disc_region(HWND(hv as _), size.width, size.height);
        }
        tauri::WindowEvent::Focused(false) => {
            if now_ms().saturating_sub(LAST_OPEN.load(Ordering::Relaxed)) > 400 {
                let _ = w.hide();
            }
        }
        _ => {}
    });
}

fn open_overlay(app: &AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    LAST_OPEN.store(now_ms(), Ordering::Relaxed);
    let _ = win.show();
    let _ = win.center();
    let _ = win.set_focus();
    if let Ok(hwnd) = win.hwnd() {
        let hv = hwnd.0 as isize;
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(90));
            let _ = app.run_on_main_thread(move || activate_overlay(HWND(hv as _)));
        });
    }
    let _ = app.emit("overlay-open", ());
}

fn lang_of(app: &AppHandle) -> Lang {
    app.state::<Mutex<settings::Settings>>().lock().unwrap().lang()
}

fn ui_config_payload(lang: Lang) -> UiConfig {
    UiConfig {
        lang: match lang {
            Lang::En => "en",
            Lang::Vi => "vi",
        },
        hint_input: lang.hint_input(),
        hint_idle: lang.hint_idle(),
        hint_keys: lang.hint_keys(),
        killing: lang.killing(),
        all_label: lang.all_label(),
        all_sub: lang.all_sub(),
        core_label: lang.core_label(),
        query_failed: lang.query_failed(),
        kill_failed: lang.kill_failed(),
        mode_port: lang.mode_port(),
        mode_proc: lang.mode_proc(),
        mode_proc_sub: lang.mode_proc_sub(),
        mode_hung: lang.mode_hung(),
        mode_hung_sub: lang.mode_hung_sub(),
    }
}

#[derive(serde::Serialize, Clone)]
struct UiConfig {
    lang: &'static str,
    hint_input: &'static str,
    hint_idle: &'static str,
    hint_keys: &'static str,
    killing: &'static str,
    all_label: &'static str,
    all_sub: &'static str,
    core_label: &'static str,
    query_failed: &'static str,
    kill_failed: &'static str,
    mode_port: &'static str,
    mode_proc: &'static str,
    mode_proc_sub: &'static str,
    mode_hung: &'static str,
    mode_hung_sub: &'static str,
}

#[derive(serde::Serialize)]
struct QueryOut {
    mode: Option<String>,
    summary: Option<String>,
    rows: Vec<proc::ProcInfo>,
}

fn query_blocking(input: String) -> QueryOut {
    let lang: Lang = settings::Settings::load().lang();
    let Some(cmd) = commands::parse(&input) else {
        return QueryOut {
            mode: None,
            summary: None,
            rows: Vec::new(),
        };
    };
    let (mode, summary, rows) = match &cmd {
        Command::Port(port) => {
            let pids = port::listeners_on_port(*port);
            let rows = proc::enrich(&pids);
            let s = lang.summary_on_port(rows.len(), *port);
            ("port".to_string(), s, rows)
        }
        Command::Name(name) => {
            let rows = proc::find_by_name(name);
            let total_mb: u64 = rows.iter().map(|p| fmt_mb(p.ram_bytes)).sum();
            let s = lang.summary_named(rows.len(), total_mb);
            ("name".to_string(), s, rows)
        }
        Command::Hang => {
            let rows = hang::hung_processes();
            let s = lang.summary_hung(rows.len());
            ("hang".to_string(), s, rows)
        }
    };
    QueryOut {
        mode: Some(mode),
        summary: Some(summary),
        rows,
    }
}

fn fmt_mb(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

fn kill_toast_text(lang: Lang, cmd: Option<Command>, report: &KillReport) -> (String, bool) {
    let ok = !report.killed.is_empty() && report.denied.is_empty() && report.failed.is_empty();
    let text = if report.killed.is_empty()
        && report.denied.is_empty()
        && report.failed.is_empty()
    {
        lang.toast_nothing().to_string()
    } else if report.killed.is_empty() {
        if let Some((_, msg)) = report.failed.first() {
            lang.toast_failed(msg)
        } else if let Some((pid, name)) = report.denied.first() {
            lang.toast_partial(0, 1, 0) + &format!(" — {name} (PID {pid})")
        } else {
            lang.toast_nothing().to_string()
        }
    } else {
        match cmd {
            Some(Command::Port(port))
                if report.killed.len() == 1
                    && report.denied.is_empty()
                    && report.failed.is_empty() =>
            {
                let (pid, name) = &report.killed[0];
                lang.toast_port_freed(port, name, *pid)
            }
            Some(Command::Port(_)) => {
                lang.toast_partial(report.killed.len(), report.denied.len(), report.failed.len())
            }
            Some(Command::Name(name)) => lang.toast_killed(report.killed.len(), &name),
            Some(Command::Hang) => lang.toast_killed_hung(report.killed.len()),
            None => lang.toast_partial(
                report.killed.len(),
                report.denied.len(),
                report.failed.len(),
            ),
        }
    };
    (text, ok)
}

#[tauri::command]
async fn query(input: String) -> Result<QueryOut, String> {
    tauri::async_runtime::spawn_blocking(move || query_blocking(input))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn kill(app: AppHandle, input: String, pids: Vec<u32>) -> Result<KillReport, String> {
    let lang = lang_of(&app);
    let cmd = commands::parse(&input);
    let force = matches!(cmd, Some(Command::Hang));
    if pids.is_empty() {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.hide();
        }
        return Ok(KillReport::default());
    }
    let report = tauri::async_runtime::spawn_blocking(move || proc::kill_pids(&pids, force))
        .await
        .map_err(|e| e.to_string())?;
    let (text, ok) = kill_toast_text(lang, cmd, &report);
    if ok {
        toast::show(&text, true);
    }
    if !report.killed.is_empty() {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.hide();
        }
    }
    Ok(report)
}

#[tauri::command]
fn ui_config(app: AppHandle) -> UiConfig {
    ui_config_payload(lang_of(&app))
}

#[tauri::command]
fn hide_overlay(win: WebviewWindow) {
    let _ = win.hide();
}

fn handle_tray(app: &AppHandle, cmd: TrayCmd) {
    let settings = app.state::<Mutex<settings::Settings>>();
    match cmd {
        TrayCmd::Exit => {
            app.exit(0);
        }
        TrayCmd::ToggleLang => {
            let (lang, autostart) = {
                let mut s = settings.lock().unwrap();
                s.lang = match s.lang {
                    settings::LangCode::En => settings::LangCode::Vi,
                    settings::LangCode::Vi => settings::LangCode::En,
                };
                s.save();
                (s.lang(), s.autostart)
            };
            TRAY.with(|t| {
                if let Some(t) = t.borrow().as_ref() {
                    t.lang_item.set_text(lang.menu_lang_switch_hint());
                    t.autostart_item.set_text(lang.menu_autostart(autostart));
                    t.exit_item.set_text(lang.menu_exit());
                }
            });
            let _ = app.emit("lang-changed", ui_config_payload(lang));
        }
        TrayCmd::ToggleAutostart => {
            let (new, lang) = {
                let mut s = settings.lock().unwrap();
                let new = !s.autostart;
                match settings::set_autostart(new) {
                    Ok(()) => {
                        s.autostart = new;
                        s.save();
                        (Ok(()), s.lang())
                    }
                    Err(e) => (Err(e), s.lang()),
                }
            };
            match new {
                Ok(()) => {
                    let on = settings.lock().unwrap().autostart;
                    TRAY.with(|t| {
                        if let Some(t) = t.borrow().as_ref() {
                            t.autostart_item.set_text(lang.menu_autostart(on));
                        }
                    });
                    toast::show(&lang.menu_autostart(on), true);
                }
                Err(e) => toast::show(&lang.toast_failed(&e), false),
            }
        }
    }
}

fn spawn_dispatcher(app: AppHandle, rx: Receiver<Event>) {
    std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() {
            match ev {
                Event::Hotkey => {
                    let a = app.clone();
                    let _ = app.run_on_main_thread(move || open_overlay(&a));
                }
                Event::HotkeyFailed(msg) => {
                    eprintln!("PortZap: {msg}");
                    let lang = lang_of(&app);
                    let wide: Vec<u16> = lang.hotkey_taken().encode_utf16().collect();
                    let title: PCWSTR = windows::core::w!("PortZap");
                    unsafe {
                        MessageBoxW(
                            None,
                            PCWSTR(wide.as_ptr()),
                            title,
                            MB_OK | MB_ICONWARNING,
                        );
                    }
                }
                Event::Cmd(cmd) => {
                    let a = app.clone();
                    let _ = app.run_on_main_thread(move || handle_tray(&a, cmd));
                }
            }
        }
    });
}

/// Phải khớp `identifier` trong tauri.conf.json — WebView2 profile nằm ở
/// %LOCALAPPDATA%\<identifier>\EBWebView. WebView2 cache asset theo URL, nên
/// rebuild UI mà không xóa cache sẽ vẫn hiện giao diện cũ ("sửa file không ăn").
const WEBVIEW_IDENTIFIER: &str = "com.portzap.app";

fn invalidate_webview_cache() {
    let Ok(local) = std::env::var("LOCALAPPDATA") else {
        return;
    };
    let root = std::path::Path::new(&local).join(WEBVIEW_IDENTIFIER);
    let marker = root.join("profile_fingerprint");
    let current = format!("{}-{}", env!("CARGO_PKG_VERSION"), UI_FINGERPRINT.trim());
    if std::fs::read_to_string(&marker).map(|p| p == current).unwrap_or(false) {
        return;
    }
    let _ = std::fs::remove_dir_all(root.join("EBWebView"));
    let _ = std::fs::create_dir_all(&root);
    let _ = std::fs::write(&marker, current);
}

fn main() {
    invalidate_webview_cache();
    let settings = settings::Settings::load();
    let (tx, rx) = mpsc::channel::<Event>();
    let bus = Bus { tx };

    hotkey::spawn(bus.clone());

    tauri::Builder::default()
        .manage(Mutex::new(settings))
        .setup(move |app| {
            let (lang, autostart) = {
                let s = app.state::<Mutex<settings::Settings>>();
                let s = s.lock().unwrap();
                (s.lang(), s.autostart)
            };
            if let Some(win) = app.get_webview_window("main") {
                if let Ok(size) = win.inner_size() {
                    if let Ok(hwnd) = win.hwnd() {
                        set_disc_region(hwnd, size.width, size.height);
                    }
                }
                arm_window_events(&win);
            }
            let tray = tray::init(bus, lang, autostart);
            TRAY.with(|t| *t.borrow_mut() = Some(tray));
            spawn_dispatcher(app.handle().clone(), rx);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            query,
            kill,
            ui_config,
            hide_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running PortZap");
}
