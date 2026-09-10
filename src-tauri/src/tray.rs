// Tray icon + menu, mirroring the WinForms `notifyIcon1` / `contextMenuStrip1`.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, Wry};

use crate::commands;

pub fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    // Restores a minimized window, mirroring how the original app is
    // brought back after the user minimizes it to keep the app resident.
    let show_window = MenuItem::with_id(app, "show_window", "ウィンドウを表示", true, None::<&str>)?;
    let copy_unixtime = MenuItem::with_id(
        app,
        "copy_unixtime",
        "UnixTime値をコピー",
        true,
        None::<&str>,
    )?;
    let copy_ymd1 = MenuItem::with_id(
        app,
        "copy_ymd1",
        "Y/m/d H:i:sをコピー",
        true,
        None::<&str>,
    )?;
    let copy_ymd2 = MenuItem::with_id(app, "copy_ymd2", "YmdHisをコピー", true, None::<&str>)?;
    let exit_item = MenuItem::with_id(app, "exit", "終了(&C)", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_window,
            &PredefinedMenuItem::separator(app)?,
            &copy_unixtime,
            &copy_ymd1,
            &copy_ymd2,
            &PredefinedMenuItem::separator(app)?,
            &exit_item,
        ],
    )?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("TimeCopyIconRust")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let app = app.clone();
            match event.id().as_ref() {
                "show_window" => show_main_window(&app),
                "copy_unixtime" => {
                    let _ = commands::action_copy_unixtime(app);
                }
                "copy_ymd1" => {
                    let _ = commands::action_copy_ymd1(app);
                }
                "copy_ymd2" => {
                    let _ = commands::action_copy_ymd2(app);
                }
                "exit" => {
                    commands::action_exit(app);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                // Plain left click -> show/restore the main window (e.g.
                // after it was minimized). `show_menu_on_left_click` is
                // disabled below so a left click is free for this instead.
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    show_main_window(tray.app_handle());
                }
                // Double-click the tray icon -> copy current UnixTime
                // (mirrors `notifyIcon1_DoubleClick`). NOTE: tray-icon's
                // `DoubleClick` event is Windows-only; macOS has no
                // equivalent, so this shortcut is unavailable there (see
                // docs/PORTING_NOTES.md). The preceding `Click` event above
                // will still have shown the window either way.
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    let app = tray.app_handle().clone();
                    let _ = commands::action_copy_unixtime(app);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

/// Show and focus the main window. Used both when the tray/dock icon is
/// activated and when a second app instance is launched (see
/// `tauri_plugin_single_instance` registration in `lib.rs`).
pub fn show_main_window(app: &tauri::AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
