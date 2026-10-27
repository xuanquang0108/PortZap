# Spec: PortZap — Lưỡi hái giải phóng tài nguyên Windows

## Objective
Công cụ nền trên Windows: nhấn **Alt+K** → vòng tròn (ring) spinner hiện giữa màn hình → gõ lệnh → Enter → kill & báo toast. Mục tiêu hiệu suất: **< 100ms** từ lúc bấm hotkey đến lúc hiện UI. Không tìm file, không tính toán rườm rà.

**User flow:**
```
Alt+K → Ring spinner giữa màn hình (có ô nhập)
  ├─ ":3000"  → list process giữ port 3000 (tên, PID, RAM) → Enter → kill → toast "Port 3000 freed (PID 14205)"
  ├─ "node"   → list node.exe (số lượng, tổng RAM) → Enter → kill tất cả → toast
  └─ "hang"   → list app Not Responding → Enter → force kill → toast
Esc → đóng overlay. Click tray → Exit / đổi ngôn ngữ / bật-tắt autostart.
```

## Tech Stack & Commands
- Rust (edition 2024) + crates: `windows` (RegisterHotKey, GetExtendedTcpTable, IsHungAppWindow, Shell_NotifyIcon APIs), `eframe`/`egui` (overlay ring, transparent, frameless), `sysinfo` (RAM/CPU), `tray-icon` (tray menu), `serde`/`serde_json` (settings)

```
Build:   cargo build --release
Test:    cargo test
Lint:    cargo clippy
Run:     cargo run
```

## Project Structure
```
src/main.rs      → entry, eframe app, UserEvent (Hotkey/KillDone/Tray), wire mọi module
src/hotkey.rs    → hotkey thread riêng (message-only window + RegisterHotKey Alt+K)
src/overlay.rs   → ring spinner + ô nhập + kết quả (egui, centered, always-on-top, transparent)
src/commands.rs  → parse lệnh: ":3000" | "tên_app" | "hang"/"frozen"
src/port.rs      → tìm PID giữ port (GetExtendedTcpTable IPv4+IPv6, TCP listener)
src/proc.rs      → liệt kê tiến trình theo tên + RAM/CPU, kill graceful→force (taskkill /T → /F /T)
src/hang.rs      → EnumWindows + IsHungAppWindow → PID app treo
src/toast.rs     → toast mini Win32 góc phải dưới, tự tắt ~2.8s, variant ok/error
src/tray.rs      → tray icon: Exit, Language (en/vi), Autostart on/off
src/settings.rs  → %APPDATA%\portzap\settings.json (lang, autostart)
src/i18n.rs      → chuỗi EN/VI cho mọi thông báo
```

## Behavior cốt lõi
| Trạng thái | Hành vi |
|---|---|
| Live results | Gõ lệnh → hiện kết quả ngay khi gõ (không cần Enter) |
| Kill graceful→force | `taskkill /PID n /T` → chờ 1.5s → còn sống? → `taskkill /F /T`. Riêng `hang`: force thẳng ngay |
| Thiếu quyền | Kill fail (access denied) → toast "Cần quyền Admin" / "Access denied — run as admin", không crash |
| Autostart | Registry HKCU Run, mặc định OFF, bật/tắt được từ tray |
| Ngôn ngữ | EN/VI切换 từ tray, lưu vào settings |
| Hotkey | Alt+K cố định trong MVP; nếu đã bị chiếm → MessageBox báo lỗi khi khởi động |

## Testing Strategy
- Unit test: parse lệnh (`:3000`, `node`, `hang`, input rỗng/sai), settings default
- Manual checklist: hotkey <100ms, kill port thật (dev server), kill theo tên, kill app treo, access denied, Esc/Exit, EN/VI, autostart toggle

## Boundaries
- **Always:** `cargo test` + `cargo clippy` trước khi xong task
- **Ask first:** thêm crate mới, đổi hotkey mặc định
- **Never:** kill process System/PID 4, commit file settings chứa path người dùng

## Success Criteria
1. Alt+K hiện ring < 100ms
2. 3 lệnh kill hoạt động thật trên Windows thật
3. Toast hiện đúng PID/tên sau mỗi lần kill
4. `cargo test` + `cargo clippy` pass
