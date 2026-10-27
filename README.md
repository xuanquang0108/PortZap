<div align="center">
  <img src="icons/icon.png" width="112" alt="PortZap icon">

  <h1>PortZap</h1>

  <p><b>Kill whatever is holding a port, without leaving the keyboard.</b></p>
  <p>Press <kbd>Alt</kbd>+<kbd>K</kbd>, type <code>:3000</code>, press <kbd>Enter</kbd>. The port is free.</p>

  <p>
    <a href="../../releases/latest"><img src="https://img.shields.io/github/v/release/xuanquang0108/PortZap?label=release" alt="Latest release"></a>
    <a href="../../actions/workflows/release.yml"><img src="https://github.com/xuanquang0108/PortZap/actions/workflows/release.yml/badge.svg" alt="Release workflow status"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-16a34a" alt="MIT license"></a>
  </p>

  <p>Windows 10/11 x64 · Portable · English &amp; Tiếng Việt · MIT</p>
</div>

<p align="center">
  <img src="docs/screenshot.png" width="480" alt="PortZap listing three processes that hold port 3000, with an ALL button to kill them">
</p>

<p align="center">
  <img src="docs/screenshot-idle.png" width="300" alt="PortZap idle: PORT, PROCESS and HUNG shortcuts around an empty input">
</p>

## Why

A dev server dies with `EADDRINUSE`. Finding the culprit means opening Task Manager and
scanning the Details tab, or running `netstat -ano | findstr :3000` and then
`taskkill`. Three windows, none of them where you were working.

PortZap collapses that into three keystrokes. It lives in the system tray, opens an
overlay in the centre of the screen on <kbd>Alt</kbd>+<kbd>K</kbd>, and kills the
processes you select while your editor stays in focus.

Speed is the point. The overlay targets **under 100 ms** from keypress to visible:
it reads the Windows connection table directly (`GetExtendedTcpTable`) and never
scans the disk.

## What you get

- **`:3000`** — every process holding port 3000: name, PID, memory. <kbd>Enter</kbd> kills them.
- **`node`** — every process whose name matches, killed in one pass.
- **`hang`** — apps stuck at *Not Responding*, force-killed.
- **Mode is inferred.** Type a number or a name; PortZap works out which query you meant.
- **A toast after each kill** — how many died, which were refused for lack of Admin rights.
- **Language switch in the tray menu** — English / Tiếng Việt, applied immediately.
- **Start with Windows** — off by default, one click in the tray menu.
- **No taskbar entry.** The overlay appears when you call it and disappears when you don't.

## Use it

| Key | Does |
| --- | --- |
| <kbd>Alt</kbd>+<kbd>K</kbd> | Open or close the overlay |
| <kbd>Enter</kbd> | Kill the listed processes |
| <kbd>Esc</kbd> | Close the overlay |

| Query | Returns |
| --- | --- |
| `:3000` or `3000` | Processes holding port 3000 |
| `node`, `chrome`, `Code`, … | Processes matching that name |
| `hang` | Apps that are not responding |

Right-click the tray icon for **Language**, **Start with Windows**, **Exit**.

## Download

Grab the zip from [**Releases**](../../releases/latest), unpack it, run `portzap.exe`.
No installer, no leftovers: the only registry key it ever writes is the autostart entry
you switch on yourself.

- Windows 10/11 x64 · [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (bundled with Windows 11; separate install on older Windows 10)
- Settings live in `%APPDATA%\portzap\settings.json` (language and autostart)
- Smart App Control may refuse to run unsigned executables. If that happens, build from
  source below.

## Build from source

You need [Rust 1.90](https://rustup.rs/) (pinned in `rust-toolchain.toml`). Node.js 20+
is only needed if you touch the UI.

```powershell
git clone https://github.com/xuanquang0108/PortZap.git
cd PortZap

cargo build --release      # ui/dist is committed, Node is not required
target\release\portzap.exe
```

> If `cargo build --release` fails with *An Application control policy has blocked this
> file* (`os error 4551`), Smart App Control is in enforcement mode on your machine and
> is rejecting unsigned binaries cargo just produced. Debug builds keep working, and
> official releases are built by CI instead — see `RELEASING.md`.

After changing `ui/index.html`, `ui/app.js` or `ui/input.css`, rebuild the UI:

```powershell
cd ui
npm install        # first time only
npm run build      # writes ui/dist/ — or npm run watch while developing
```

Check your work:

```powershell
cargo test
cargo clippy --all-targets
```

> **Rule:** never edit files in `ui/dist/` by hand. They are generated from `ui/`,
> compiled straight into the exe, and committed alongside the source. Edit the source,
> run `npm run build`, commit both.

## Project layout

```
├── src/
│   ├── main.rs        entry point, overlay window, wiring
│   ├── hotkey.rs      Alt+K registration (RegisterHotKey)
│   ├── port.rs        processes holding a port (GetExtendedTcpTable)
│   ├── proc.rs        process lookup and kill (taskkill /F /T)
│   ├── hang.rs        detection of unresponsive apps
│   ├── commands.rs    query parsing — port, name, or hang
│   ├── tray.rs        tray icon and menu (icon drawn in code)
│   ├── toast.rs       Windows toast notifications
│   ├── settings.rs    settings.json and autostart (registry Run key)
│   └── i18n.rs        English / Tiếng Việt strings
├── ui/
│   ├── index.html     overlay markup
│   ├── app.js         petals, results, Tauri IPC
│   ├── input.css      palette (Tailwind 4 + CSS variables)
│   └── dist/          generated — do not edit
├── icons/             app icon (multi-size .ico, .png)
├── make_icon.ps1      regenerates icons/
├── tauri.conf.json    window and app configuration
└── RELEASING.md       how a version gets shipped
```

## Change the look

- **Colour:** edit `ui/input.css` (`--acc`, `--acc-bright`, `--tint`, `--rim`, `--bg`),
  then `npm run build`.
- **Icon:** edit the drawing code in `make_icon.ps1` and run
  `powershell -ExecutionPolicy Bypass -File make_icon.ps1`. The tray icon is drawn from
  the same shape in `src/tray.rs`.

## Contributing

Fork, branch, change, and check before opening a PR:

```powershell
npm run build      # if you touched the UI
cargo test
cargo clippy --all-targets
```

Bugs and ideas go in [Issues](../../issues).

## License

[MIT](LICENSE) © PortZap contributors.
