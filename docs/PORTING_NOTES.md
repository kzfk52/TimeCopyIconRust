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
  tray-only startup.
- **Out-of-range unixtime input**: `DateTimeOffset.FromUnixTimeSeconds` in
  the original throws an unhandled exception for out-of-range values. The
  Rust port returns `None` (empty output field) instead — a safety
  improvement, not a faithfulness gap.
- **Window close button — matches the original, not a change**: an earlier
  draft of this port had the × / titlebar close button hide the window
  instead of quitting (a "tray-resident app" pattern this port initially
  assumed, without actually checking the original's behavior). The app
  owner confirmed this was wrong: in the original, `FormClosing` never
  cancels the close (it only hides the tray icon first), so × always fully
  exits the app; minimizing is how the user keeps it running in the tray
  when it's in the way. The port now matches this exactly — no
  `CloseRequested` override at all, so × uses Tauri's default
  close-then-exit-when-no-windows-remain behavior. The tray menu's
  "ウィンドウを表示" item (and a plain left-click on the tray icon) is kept
  as an added convenience for restoring a *minimized* window, since the
  original has no equivalent restore-from-tray shortcut.

## Added features (not present in the original)

- **Window position/size/maximized-state persistence**: the original has no
  `Settings.settings` / registry-based persistence of any kind — every
  launch starts at the WinForms designer's fixed location and size. This
  port remembers window position, size, and maximized state across
  restarts via `tauri-plugin-window-state`, requested by the app owner as
  a new feature. Saved to `.window-state.json` in the app's config
  directory, written on app exit (`RunEvent::Exit`, which still fires
  after `commands::action_exit`'s explicit `destroy()` calls). Only
  `StateFlags::POSITION | SIZE | MAXIMIZED` are restored — `VISIBLE` is
  intentionally excluded so this doesn't interact with the
  always-visible-on-startup behavior above.

## Known platform limitations

- **Windows exit hang (fixed)**: calling `app.exit(0)` from the "終了(&C)"
  tray menu item while a window still existed (this was originally found
  while the window was hidden, under the now-reverted hide-on-close design
  above, but the underlying risk isn't specific to hidden windows) caused
  Windows to log `Failed to unregister class Chrome_WidgetWin_0. Error =
  1412` (`ERROR_CLASS_HAS_WINDOWS`) and the process did not fully
  terminate. Reported by the app owner testing a Windows build (`npm run
  app:dev`). Fixed by explicitly calling `WebviewWindow::destroy()` (which
  skips `CloseRequested` entirely, unlike `close()`) on every window before
  `app.exit(0)` in `commands::action_exit`. This is kept even though ×
  no longer hides the window, since the tray "終了" item can still be used
  while the window is open or minimized.
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
- **Windows** (`npm run app:dev`, real hardware, by the app owner): tray
  icon does appear and is usable. Two rounds of feedback from this testing
  led to fixes: the (now-reverted) hide-on-close design stranding the
  window, and the exit hang (both documented above under "Known platform
  limitations"), plus reverting × to fully quit like the original.

## Not yet verified (requires real hardware)

- macOS: tray icon, OS notifications, single-instance focus-existing-window,
  Dock icon hiding (`ActivationPolicy::Accessory`), and the "show window"
  tray menu item / left-click restore-from-minimized behavior.
- Windows: OS notification (toast/banner) display, and re-testing the full
  close(=quit)/minimize/restore-from-tray/tray-exit flow after the latest
  round of fixes.
- `cargo tauri build` release bundling (with installers) on both platforms.
- Window position/size/maximized-state persistence across restarts
  (`tauri-plugin-window-state`) on both Windows and macOS, and multi-monitor
  behavior when a saved position's monitor has since been disconnected.
