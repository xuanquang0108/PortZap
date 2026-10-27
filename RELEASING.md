# Quy trình phát hành (Release)

Mục tiêu mỗi lần phát hành: **1 tag Git + 1 bản trên GitHub Releases** chứa file zip
portable (giải nén là chạy).

**Bản phát hành build bằng CI** (GitHub Actions) — không build local, vì máy dev bật
Smart App Control (enforcement) chặn file PE build mới chưa ký (`os error 4551`).

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
   - Tag phải trùng: `v0.1.0` (CI sẽ kiểm tra và fail nếu lệch).
2. **Sửa UI** (nếu có) → chạy lại dist:
   ```powershell
   cd ui; npm run build; cd ..
   ```
3. **Kiểm thử**:
   ```powershell
   cargo test
   cargo clippy --all-targets
   ```
4. **Chạy thử tay** bản debug `target\debug\portzap.exe` — mở overlay, kill 1 port thật.

---

## 2. Commit + tag

```powershell
git status                      # kiểm tra chỉ có thay đổi mong muốn
git add -A
git commit -m "release: v0.1.0"
git tag -a v0.1.0 -m "PortZap v0.1.0"
```

---

## 3. Push (CI làm phần còn lại)

```powershell
git push origin main
git push origin v0.1.0
```

Push tag `v*` kích hoạt workflow `.github/workflows/release.yml`, nó sẽ:

1. Kiểm tra version đồng bộ (`tag` ↔ `Cargo.toml` ↔ `tauri.conf.json`)
2. `cargo test`
3. `scripts/package.ps1` → `cargo build --release` + zip
   `assets/PortZap-v<version>-win-x64.zip`
4. Đăng zip vào **GitHub Releases** của tag đó (kèm release notes tự sinh)

Xem tiến trình: **Actions → release → tag vừa push**. Tải artifact trực tiếp cũng được
(tại trang run → *Artifacts*).

---

## 4. Build local (tùy chọn, khi không muốn chờ CI)

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package.ps1
```

Script sẽ: `cargo build --release` → gom `portzap.exe` + `README.md` + `LICENSE`
→ tạo `assets\PortZap-v<version>-win-x64.zip`.

> Thư mục `assets/` nằm trong `.gitignore` — artifact không vào Git.

**Nếu bị chặn (`An Application Control policy has blocked this file`, `os error 4551`)**:
máy bạn bật Smart App Control ở chế độ enforcement. Thử lần nữa (một số file qua được),
hoặc build trên CI. Tắt SAC thì **không bật lại được** nếu không reset Windows.

---

## 5. Đăng tay (fallback nếu CI không chạy)

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

- [ ] Actions run `release` xanh (không đỏ)
- [ ] Mở link [Releases](../../releases) → file zip tải được
- [ ] README (mục **Tải về**) → link `../../releases/latest` trỏ đúng bản mới
- [ ] Giải nén zip trên 1 máy sạch → chạy được

---

## Sau này muốn làm thêm

- **Trình cài đặt (NSIS/MSI)**: bật `bundle.active: true` trong `tauri.conf.json`,
  chạy `npm i -g @tauri-apps/cli && tauri icon icons/icon.png` để sinh bộ icon chuẩn,
  rồi `tauri build` (CI cần cài Node để chạy được).
- **CI cho PR**: thêm workflow chạy `cargo test` + `cargo clippy` trên mỗi push/PR.
