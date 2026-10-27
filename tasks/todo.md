# Todo — PortZap MVP

- [x] Task 1: Thêm crates + scaffold module rỗng
  - Acceptance: Cargo.toml đủ deps, module files tồn tại, `cargo build` pass
  - Verify: `cargo build` ✓
  - Files: Cargo.toml, src/*.rs
- [x] Task 2: Hotkey thread Alt+K → UserEvent
  - Acceptance: bấm Alt+K → eframe nhận Event::Hotkey
  - Verify: app chạy ổn định sau khi gửi Alt+K ✓ (UI cần test tay)
  - Files: src/hotkey.rs, src/main.rs
- [x] Task 3: Overlay ring + ô nhập + Esc
  - Acceptance: Alt+K → ring spinner giữa màn hình + ô nhập; Esc đóng
  - Verify: cần test tay (UI)
  - Files: src/overlay.rs, src/main.rs
- [x] Task 4: commands.rs parse + unit tests
  - Acceptance: parse đúng; test pass
  - Verify: `cargo test` 6/6 ✓
  - Files: src/commands.rs
- [x] Task 5: port.rs tìm PID theo port
  - Acceptance: `:3000` hiện đúng PID (GetExtendedTcpTable IPv4/IPv6)
  - Verify: unit test parse_rows ✓ + manual cần tay
  - Files: src/port.rs
- [x] Task 6: proc.rs kill graceful→force + denied
  - Acceptance: kill graceful→force; denied trả lỗi rõ; tree kill (/T)
  - Verify: code hoàn chỉnh, cần manual kill process thật
  - Files: src/proc.rs
- [x] Task 7: Flow `:3000` end-to-end
  - Acceptance: gõ `:3000` → list → Enter → kill → toast → overlay đóng
  - Verify: cần test tay
  - Files: src/overlay.rs, src/main.rs
- [x] Task 8: Kill theo tên + RAM/CPU
  - Acceptance: gõ `node` → list + tổng RAM; Enter kill tất cả
  - Verify: cần test tay
  - Files: src/proc.rs, src/overlay.rs
- [x] Task 9: hang.rs — IsHungAppWindow
  - Acceptance: `hang` → list app Not Responding; Enter force kill
  - Verify: cần test tay
  - Files: src/hang.rs
- [x] Task 10: toast.rs Win32 mini window
  - Acceptance: toast ok/error góc phải dưới, tự tắt ~2.8s
  - Verify: code hoàn chỉnh, cần test tay
  - Files: src/toast.rs
- [x] Task 11: Tray + settings + autostart
  - Acceptance: tray Exit; lang toggle; autostart toggle (HKCU Run, default OFF); settings JSON
  - Verify: cần test tay
  - Files: src/tray.rs, src/settings.rs, src/main.rs
- [x] Task 12: Wire i18n EN/VI
  - Acceptance: mọi chuỗi theo lang đang chọn
  - Verify: code hoàn chỉnh, cần test tay
  - Files: src/i18n.rs, src/overlay.rs, src/main.rs
- [x] Task 13: Final: clippy + test + checklist
  - Acceptance: clippy không warning, test pass, release build OK
  - Verify: `cargo clippy` ✓ `cargo test` 6/6 ✓ release exe 5.2MB chạy được ✓
  - Files: —

## Ghi chú build (Smart App Control)
Máy này bật Smart App Control → chặn build script của rustc ≥1.95.
Giải pháp đang dùng: `rust-toolchain.toml` pin rustc **1.90.0** + eframe/egui 0.33 + sysinfo 0.38.
Build bằng `cargo build` trong thư mục project (tự dùng 1.90.0).
