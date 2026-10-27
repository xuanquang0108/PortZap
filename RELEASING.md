# Quy trình phát hành (Release)

Mục tiêu mỗi lần phát hành: **1 tag Git + 1 bản trên GitHub Releases** chứa file zip
portable (giải nén là chạy).

---

## 0. Chuẩn bị (chỉ làm 1 lần)

```powershell
gh auth status          # xem đã đăng nhập GitHub CLI chưa
gh auth login           # nếu chưa → GitHub.com → HTTPS → Login with a web browser
```

Chưa có repo remote thì tạo luôn (chỉ làm 1 lần):

```powershell
gh repo create PortZap --public --source=. --remote=origin --push
```

---

## 1. Chuẩn bị code

1. **Bump version** — 2 chỗ phải trùng nhau:
   - `Cargo.toml` → `version = "0.1.0"`
   - `tauri.conf.json` → `"version": "0.1.0"`
2. **Sửa UI** (nếu có) → chạy lại dist:
   ```powershell
   cd ui; npm run build; cd ..
   ```
3. **Kiểm thử**:
   ```powershell
   cargo test
   cargo clippy --all-targets
   ```
4. **Chạy thử tay** `target\release\portzap.exe` — mở overlay, kill 1 port thật.

---

## 2. Build + đóng gói

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package.ps1
```

Script sẽ: `cargo build --release` → gom `portzap.exe` + `README.md` + `LICENSE`
→ tạo `assets\PortZap-v<version>-win-x64.zip`.

> Thư mục `assets/` nằm trong `.gitignore` — artifact không vào Git.

---

## 3. Commit + tag

```powershell
git status                      # kiểm tra chỉ có thay đổi mong muốn
git add -A
git commit -m "release: v0.1.0"
git tag -a v0.1.0 -m "PortZap v0.1.0"
```

---

## 4. Push

```powershell
git push origin main
git push origin v0.1.0
```

---

## 5. Đăng lên GitHub Releases

Cách nhanh (GitHub CLI):

```powershell
gh release create v0.1.0 "assets\PortZap-v0.1.0-win-x64.zip" `
  --title "PortZap v0.1.0" `
  --notes-file RELEASE_NOTES.md
```

Cách thủ công: mở repo trên GitHub → **Releases → Draft a new release** →
chọn tag `v0.1.0` → kéo file zip vào → viết mô tả → **Publish release**.

### Mẫu `RELEASE_NOTES.md`

```markdown
## PortZap v0.1.0

Bản đầu tiên.

### Tải về
Giải nén `PortZap-v0.1.0-win-x64.zip` rồi chạy `portzap.exe` — không cần cài đặt.

### Tính năng
- Alt+K mở đĩa lệnh giữa màn hình
- Tra tiến trình giữ port (`:3000`), theo tên (`node`), app treo (`hang`)
- Menu tray: đổi ngôn ngữ (VI/EN), khởi động cùng Windows
- Toast báo kết quả, hỗ trợ quyền Admin khi bị từ chối

### Biết rồi
- Windows 10/11 x64, cần WebView2
- Smart App Control có thể chặn exe chưa ký → build lại từ source nếu bị
```

---

## 6. Checklist sau khi đăng

- [ ] Mở link [Releases](../../releases) → file zip tải được
- [ ] README (mục **Tải về**) → link `../../releases/latest` trỏ đúng bản mới
- [ ] Giải nén zip trên 1 máy sạch → chạy được

---

## Sau này muốn làm thêm

- **Trình cài đặt (NSIS/MSI)**: bật `bundle.active: true` trong `tauri.conf.json`,
  chạy `npm i -g @tauri-apps/cli && tauri icon icons/icon.png` để sinh bộ icon chuẩn,
  rồi `tauri build`. Khi đó nên có CI (GitHub Actions) để build trên Linux/Windows.
- **CI**: thêm workflow chạy `cargo test` + `cargo clippy` trên mỗi PR.
