mod commands;
mod datetime;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single-instance must be the first plugin registered (Tauri requirement).
    // Unlike the original WinForms app, which pops a message box and exits
    // the second instance, we bring the existing window to the front —
    // the conventional Tauri UX, agreed with the user as an intentional
    // behavior change from the C# original.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window (red-button / titlebar close) hides it
            // instead of quitting, since this is a tray-resident app.
            // The "終了(&C)" menu action calls `app.exit()` directly, which
            // does not go through `CloseRequested`, so it always quits for
            // real.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::action_copy_unixtime,
            commands::action_copy_ymd1,
            commands::action_copy_ymd2,
            commands::action_exit,
            commands::action_announce_p,
            commands::convert_unixtime_input,
            commands::convert_iso8601_input,
            commands::copy_iso8601_example,
            commands::get_iso8601_example_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
