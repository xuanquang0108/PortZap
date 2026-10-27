use std::cell::RefCell;
use std::thread;

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, CreatePen, DeleteObject, DrawTextW, EndPaint,
    RoundRect, SelectObject, SetBkMode, SetTextColor, UpdateWindow, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
    FW_SEMIBOLD, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    MSG, RegisterClassW, SetTimer, ShowWindow, SystemParametersInfoW, TranslateMessage, CS_HREDRAW,
    CS_VREDRAW, SPI_GETWORKAREA, SW_SHOW, WM_DESTROY, WM_PAINT, WM_TIMER, WNDCLASSW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

const TOAST_W: i32 = 400;
const TOAST_H: i32 = 56;
const TOAST_MS: u32 = 2800;

thread_local! {
    static TOAST: RefCell<(String, bool)> = const { RefCell::new((String::new(), true)) };
}

fn colorref(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16)
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                paint(hwnd);
                LRESULT(0)
            }
            WM_TIMER => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn paint(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        if hdc.is_invalid() {
            return;
        }
        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);

        let (text, ok) = TOAST.with(|t| t.borrow().clone());
        let bg = if ok {
            colorref(22, 163, 74)
        } else {
            colorref(203, 37, 37)
        };

        let brush = CreateSolidBrush(bg);
        let pen = CreatePen(PS_SOLID, 1, bg);
        let old_brush = SelectObject(hdc, HGDIOBJ(brush.0));
        let old_pen = SelectObject(hdc, HGDIOBJ(pen.0));
        let _ = RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, 16, 16);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(HGDIOBJ(brush.0));
        let _ = DeleteObject(HGDIOBJ(pen.0));

        let font: HFONT = CreateFontW(
            -20,
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            w!("Segoe UI"),
        );
        let old_font = SelectObject(hdc, HGDIOBJ(font.0));
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colorref(255, 255, 255));
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut wide,
            &mut rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
        SelectObject(hdc, old_font);
        let _ = DeleteObject(HGDIOBJ(font.0));

        let _ = EndPaint(hwnd, &raw mut ps);
    }
}

fn bottom_right_pos() -> (i32, i32) {
    let mut rc = RECT::default();
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rc as *mut RECT as *mut _),
            windows::Win32::UI::WindowsAndMessaging::SPIF_UPDATEINIFILE,
        );
    }
    (rc.right - TOAST_W - 16, rc.bottom - TOAST_H - 16)
}

pub fn show(text: &str, ok: bool) {
    let text = text.to_string();
    thread::spawn(move || unsafe {
        TOAST.with(|t| *t.borrow_mut() = (text, ok));

        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            lpszClassName: w!("PortZapToast"),
            ..Default::default()
        };
        let _ = RegisterClassW(&class);

        let (x, y) = bottom_right_pos();
        let hwnd = match CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("PortZapToast"),
            w!(""),
            WS_POPUP,
            x,
            y,
            TOAST_W,
            TOAST_H,
            None,
            None,
            Some(hinstance),
            None,
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        let _ = SetTimer(Some(hwnd), 1, TOAST_MS, None);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}
