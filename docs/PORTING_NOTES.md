# Porting notes (TimeCopyIconWinForms -> TimeCopyIconRust)

This document tracks intentional behavior/design differences from the
original C# WinForms app, and known limitations of the Tauri port. Written
for future contributors reviewing the diff between the two apps.

## Intentional behavior changes

- **Multi-instance launch**: the original shows a message box ("既に起動して
  います。") and exits the second process. This port instead brings the
  existing window to the front (`tauri-plugin-single-instance`), matching
  standard Tauri/tray-app UX. Agreed with the app owner as an acceptable
  change.
- **Startup window state**: the original starts minimized (still visible in
  the taskbar as a minimized entry). This port instead starts with the main
  window **shown** (`visible: true` in `tauri.conf.json`) — a deliberate
  choice by the app owner, made after initially trying a fully-hidden,
  tray-only startup. The window can still be hidden via the close button
  (see "Window close button" below) and re-shown from the tray, but on
  launch it is visible rather than tray-only.
- **Out-of-range unixtime input**: `DateTimeOffset.FromUnixTimeSeconds` in
  the original throws an unhandled exception for out-of-range values. The
  Rust port returns `None` (empty output field) instead — a safety
  improvement, not a faithfulness gap.
- **Window close button**: hides the window instead of quitting (tray-resident
  app). Only the "終了(&C)" menu item calls `app.exit()` and actually quits.

## Known platform limitations

- **Tray double-click**: `tray-icon`'s `DoubleClick` event is documented as
  **Windows only**. On macOS there is no equivalent — the double-click
  "copy current UnixTime" shortcut only works on Windows. macOS users must
  use the tray menu instead.
- **Tray click conventions**: Windows convention is right-click = menu,
  double-click = default action. macOS conventionally treats left-click as
  "open the menu." `show_menu_on_left_click(false)` is set to preserve the
  Windows-style split, but macOS users may still expect left-click to open
  the menu — this is left to platform-native tray behavior rather than
  forced to match 1:1.
- **Menu bar placement**: the original's "Action" menu sits inside the form,
  just under the title bar. A native `tauri::menu::Menu` attached to the
  window would render there on Windows but jump to the top-of-screen
  application menu bar on macOS, breaking the layout parity across
  platforms. The port therefore implements "Action" as an in-page HTML/CSS
  dropdown instead of a native window menu, so the layout matches the
  screenshot on both OSes.
- **Linux/WSL tray support**: developing under WSLg, the tray icon's
  underlying D-Bus StatusNotifierItem registration fails silently
  (`GLib-GIO-CRITICAL: g_dbus_proxy_new: assertion 'G_IS_DBUS_CONNECTION
  (connection)' failed`) because WSLg has no StatusNotifierWatcher /
  notification daemon running. The app does not crash, but the tray icon is
  not visible and OS notifications are not shown. **Tray and notification
  behavior must be verified on real Windows and macOS machines**, not
  inside this WSL dev environment.

## Design fidelity notes (WinForms look vs. web look)

Full pixel-for-pixel fidelity to the original WinForms rendering is not
attempted; the following are approximated instead:

| Original (WinForms) | Port (HTML/CSS) | Note |
|---|---|---|
| Native `GroupBox` border/legend, OS-themed | `<fieldset>/<legend>` with a manually styled border | Visually close; exact border color/weight differs by OS theme in the original, fixed in the port |
| Absolute pixel coordinates + `Anchor` for partial resize | CSS Flexbox: fixed-height first group, flexible second group | Reproduces the *intent* (bottom group and its message box grow on resize), not exact pixel positions |
| Segoe UI font (Windows-only) | Font stack: `"Segoe UI", "Yu Gothic UI", "Hiragino Sans", system-ui, sans-serif` | macOS has no Segoe UI; falls back to the closest system font |
| Native `TextBox` sunken border | `box-shadow: inset ...` approximation | Close but not pixel-identical across OS themes |
| Native full-selection highlight (blue) on programmatic `.Select()` | `input.select()` + `::selection` CSS color | Approximate; not a native OS text-selection widget |

## Explicitly out of scope for this pass

- Code signing (Windows Authenticode / `signtool`, macOS `codesign`)
- macOS notarization (`notarytool`, `stapler`)
- GitHub Actions CI for Windows/macOS cross-platform builds

These are planned as follow-up work.

## Verification performed so far

- `cargo test` (in `src-tauri/`): all date/time conversion logic covered
  with fixed-timezone unit tests, including the exact numbers shown in the
  reference screenshot (`winform_screenshot.png`).
- `npm run build`: TypeScript type-checks and Vite build succeed.
- `npm run tauri dev` under WSLg: app launches and stays running (no
  crash); main window renders and responds to input; tray icon
  registration fails silently per the Linux/WSL limitation above.

## Not yet verified (requires real hardware)

- Tray icon visibility/behavior on Windows and macOS
- OS notification (toast/banner) display on Windows and macOS
- Single-instance focus-existing-window behavior on Windows and macOS
- macOS Dock icon hiding (`ActivationPolicy::Accessory`)
- `cargo tauri build` release bundling on both platforms
