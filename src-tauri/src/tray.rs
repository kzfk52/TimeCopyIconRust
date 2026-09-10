// Tray icon + menu, mirroring the WinForms `notifyIcon1` / `contextMenuStrip1`.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, Wry};

use crate::commands;

pub fn build_tray(app: &tauri::App) -> tauri::Result<()> {
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

    let menu = Menu::with_items(app, &[&copy_unixtime, &copy_ymd1, &copy_ymd2, &exit_item])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("TimeCopyIconRust")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let app = app.clone();
            match event.id().as_ref() {
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
            // Double-click the tray icon -> copy current UnixTime (mirrors
            // `notifyIcon1_DoubleClick`). NOTE: tray-icon's `DoubleClick`
            // event is Windows-only; macOS has no equivalent, so this
            // shortcut is unavailable there (see docs/PORTING_NOTES.md).
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                let _ = commands::action_copy_unixtime(app);
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
