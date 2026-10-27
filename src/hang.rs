use std::collections::HashSet;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsHungAppWindow, IsWindowVisible,
};

use crate::proc::ProcInfo;

pub fn hung_processes() -> Vec<ProcInfo> {
    let mut pids: HashSet<u32> = HashSet::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut pids as *mut HashSet<u32> as isize),
        );
    }
    let own = std::process::id();
    let mut pid_list: Vec<u32> = pids.into_iter().filter(|&p| p != own && p != 0 && p != 4).collect();
    pid_list.sort_unstable();
    crate::proc::enrich(&pid_list)
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    unsafe {
        let pids = &mut *(lparam.0 as *mut HashSet<u32>);
        if IsWindowVisible(hwnd).as_bool() && IsHungAppWindow(hwnd).as_bool() {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != 0 {
                pids.insert(pid);
            }
        }
    }
    windows::Win32::Foundation::TRUE
}
