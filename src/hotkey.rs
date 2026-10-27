use std::thread;

use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, WNDCLASSW, WM_HOTKEY, CW_USEDEFAULT, HWND_MESSAGE, MSG, WINDOW_EX_STYLE,
    WINDOW_STYLE,
};

use crate::{Bus, Event};

const HOTKEY_ID: i32 = 1;
const VK_K: u32 = 0x4B;

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn spawn(bus: Bus) {
    thread::spawn(move || unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
        let class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            lpszClassName: w!("PortZapHotkeyWnd"),
            ..Default::default()
        };
        let _ = RegisterClassW(&class);
        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("PortZapHotkeyWnd"),
            w!(""),
            WINDOW_STYLE::default(),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance),
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                bus.send(Event::HotkeyFailed(format!("hotkey window: {e}")));
                return;
            }
        };

        if let Err(e) = RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_ALT, VK_K) {
            bus.send(Event::HotkeyFailed(format!("Alt+K: {e}")));
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                bus.send(Event::Hotkey);
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);
        let _ = DestroyWindow(hwnd);
    });
}
