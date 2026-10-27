# Plan: PortZap MVP

## Thiết kế chính
- **Hotkey thread riêng**: message-only window + `RegisterHotKey(Alt+K)` → `EventLoopProxy<UserEvent>` đẩy vào eframe. An toàn với winit event loop.
- **Overlay**: eframe transparent, frameless, always-on-top spawn ở giữa màn hình chỉ khi bấm hotkey; ẩn hẳn khi đóng. Ring vẽ bằng egui Shape (arc quay), ô nhập + kết quả ngay dưới.
- **Kill pipeline**: `taskkill /T` (graceful) → 1.5s → còn sống → `taskkill /F /T`. `hang`: force thẳng. Access denied → toast đỏ.
- **Port**: `GetExtendedTcpTable` (IPv4+IPv6, listener, thuần API).
- **Toast**: cửa sổ Win32 popup riêng (thread riêng), bottom-right, tự hủy 2.8s.
- **Tray + settings**: `tray-icon` + receiver thread → UserEvent; settings JSON tại `%APPDATA%\portzap\settings.json` (lang, autostart mặc định OFF).

## Build order & Tasks
1. Thêm crates + scaffold module rỗng → `cargo build`
2. Hotkey thread Alt+K → log/UserEvent → manual
3. Overlay ring + ô nhập + Esc → manual <100ms
4. commands.rs parse + unit test → `cargo test`
5. port.rs tìm PID theo port → manual với server 3000
6. proc.rs kill graceful→force + denied → manual
7. Gộp flow `:3000` end-to-end → manual
8. Kill theo tên + RAM/CPU (sysinfo) → manual
9. hang.rs — EnumWindows + IsHungAppWindow → manual
10. toast.rs Win32 mini window → manual
11. Tray: Exit, lang, autostart toggle → manual
12. Wire i18n EN/VI → manual
13. Final: clippy sạch + checklist → `cargo clippy && cargo test`

## Rủi ro & mitigation
- winit không guarantee nhận WM_HOTKEY → hotkey thread riêng + EventLoopProxy
- window trong suốt chặn click → ẩn hẳn (`ViewportCommand::Visible(false)`) khi đóng
- sysinfo API đổi giữa các version → kiểm tra qua compile error
- HiDPI: dùng `ViewportInfo::monitor_size` (points) thay GetSystemMetrics
