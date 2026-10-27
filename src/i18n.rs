#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Vi,
}

impl Lang {
    pub fn hint_input(self) -> &'static str {
        "3000 · node"
    }

    pub fn hint_idle(self) -> &'static str {
        match self {
            Lang::En => "type a port or app name",
            Lang::Vi => "gõ cổng hoặc tên app",
        }
    }

    pub fn hint_keys(self) -> &'static str {
        match self {
            Lang::En => "Enter kill · Esc close",
            Lang::Vi => "Enter để tắt · Esc đóng",
        }
    }

    pub fn killing(self) -> &'static str {
        match self {
            Lang::En => "Killing…",
            Lang::Vi => "Đang tắt…",
        }
    }

    pub fn summary_on_port(self, n: usize, port: u16) -> String {
        match self {
            Lang::En => format!("{n} on port {port}"),
            Lang::Vi => format!("{n} tiến trình · port {port}"),
        }
    }

    pub fn summary_named(self, n: usize, total_mb: u64) -> String {
        match self {
            Lang::En => format!("{n} processes · {total_mb} MB"),
            Lang::Vi => format!("{n} tiến trình · {total_mb} MB"),
        }
    }

    pub fn summary_hung(self, n: usize) -> String {
        match self {
            Lang::En => format!("{n} app(s) not responding"),
            Lang::Vi => format!("{n} app đang treo"),
        }
    }

    pub fn toast_port_freed(self, port: u16, name: &str, pid: u32) -> String {
        match self {
            Lang::En => format!("Port {port} freed — {name} (PID {pid})"),
            Lang::Vi => format!("Đã giải phóng port {port} — {name} (PID {pid})"),
        }
    }

    pub fn toast_killed(self, n: usize, what: &str) -> String {
        match self {
            Lang::En => format!("Killed {n} {what} process(es)"),
            Lang::Vi => format!("Đã tắt {n} tiến trình {what}"),
        }
    }

    pub fn toast_killed_hung(self, n: usize) -> String {
        match self {
            Lang::En => format!("Killed {n} hung app(s)"),
            Lang::Vi => format!("Đã tắt {n} app treo"),
        }
    }

    pub fn toast_partial(self, killed: usize, denied: usize, failed: usize) -> String {
        match self {
            Lang::En => {
                let mut parts = vec![format!("Killed {killed}")];
                if denied > 0 {
                    parts.push(format!("{denied} denied (run as Admin)"));
                }
                if failed > 0 {
                    parts.push(format!("{failed} failed"));
                }
                parts.join(" · ")
            }
            Lang::Vi => {
                let mut parts = vec![format!("Đã tắt {killed}")];
                if denied > 0 {
                    parts.push(format!("{denied} bị từ chối (cần Admin)"));
                }
                if failed > 0 {
                    parts.push(format!("{failed} lỗi"));
                }
                parts.join(" · ")
            }
        }
    }

    pub fn toast_nothing(self) -> &'static str {
        match self {
            Lang::En => "Nothing to kill",
            Lang::Vi => "Không có gì để tắt",
        }
    }

    pub fn toast_failed(self, msg: &str) -> String {
        match self {
            Lang::En => format!("Kill failed: {msg}"),
            Lang::Vi => format!("Không tắt được: {msg}"),
        }
    }

    pub fn menu_lang_switch_hint(self) -> &'static str {
        match self {
            Lang::En => "Language: English (click to switch)",
            Lang::Vi => "Ngôn ngữ: Tiếng Việt (bấm để đổi)",
        }
    }

    pub fn menu_autostart(self, on: bool) -> String {
        match (self, on) {
            (Lang::En, true) => "Autostart: On".into(),
            (Lang::En, false) => "Autostart: Off".into(),
            (Lang::Vi, true) => "Khởi động cùng Windows: Bật".into(),
            (Lang::Vi, false) => "Khởi động cùng Windows: Tắt".into(),
        }
    }

    pub fn menu_exit(self) -> &'static str {
        match self {
            Lang::En => "Exit",
            Lang::Vi => "Thoát",
        }
    }

    pub fn hotkey_taken(self) -> String {
        match self {
            Lang::En => "PortZap: Alt+K is already in use by another app. The overlay may not open.".into(),
            Lang::Vi => "PortZap: Alt+K đang bị app khác dùng. Overlay có thể không mở được.".into(),
        }
    }

    pub fn all_label(self) -> &'static str {
        match self {
            Lang::En => "ALL",
            Lang::Vi => "TẤT CẢ",
        }
    }

    pub fn all_sub(self) -> &'static str {
        match self {
            Lang::En => "kill all",
            Lang::Vi => "diệt tất cả",
        }
    }

    pub fn core_label(self) -> &'static str {
        match self {
            Lang::En => "TARGET",
            Lang::Vi => "ĐÍCH",
        }
    }

    pub fn query_failed(self) -> &'static str {
        match self {
            Lang::En => "Query failed",
            Lang::Vi => "Không truy vấn được",
        }
    }

    pub fn kill_failed(self) -> &'static str {
        match self {
            Lang::En => "Couldn't kill",
            Lang::Vi => "Không tắt được",
        }
    }

    pub fn mode_port(self) -> &'static str {
        "PORT"
    }

    pub fn mode_proc(self) -> &'static str {
        match self {
            Lang::En => "PROCESS",
            Lang::Vi => "TIẾN TRÌNH",
        }
    }

    pub fn mode_proc_sub(self) -> &'static str {
        match self {
            Lang::En => "node …",
            Lang::Vi => "tên app …",
        }
    }

    pub fn mode_hung(self) -> &'static str {
        match self {
            Lang::En => "HUNG",
            Lang::Vi => "TREO",
        }
    }

    pub fn mode_hung_sub(self) -> &'static str {
        match self {
            Lang::En => "not responding",
            Lang::Vi => "không phản hồi",
        }
    }
}
