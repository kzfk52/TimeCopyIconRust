// Tauri commands invoked from the frontend. Clipboard and notification
// side effects are kept entirely on the Rust side (rather than exposing the
// clipboard-manager/notification plugin APIs to the frontend) so the
// frontend only ever asks "perform this action" and never touches the
// clipboard or OS notifications directly.

use chrono::Local;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::datetime;

fn notify(app: &AppHandle, title: &str, body: &str) {
    // Best-effort: a missing notification daemon (e.g. under WSL) should
    // never break the copy action itself.
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

fn copy_and_notify(app: &AppHandle, text: &str, notify_body: &str) -> Result<String, String> {
    app.clipboard()
        .write_text(text.to_string())
        .map_err(|e| e.to_string())?;
    notify(app, "TimeCopyIconRust", notify_body);
    Ok(text.to_string())
}

/// "UnixTime値をコピー"
#[tauri::command]
pub fn action_copy_unixtime(app: AppHandle) -> Result<String, String> {
    let text = datetime::unixtime_str_now();
    copy_and_notify(&app, &text, &format!("コピーしました: {text}"))
}

/// "Y/m/d H:i:sをコピー"
#[tauri::command]
pub fn action_copy_ymd1(app: AppHandle) -> Result<String, String> {
    let text = datetime::ymd1_str_now(&Local);
    copy_and_notify(&app, &text, &format!("コピーしました: {text}"))
}

/// "YmdHisをコピー"
#[tauri::command]
pub fn action_copy_ymd2(app: AppHandle) -> Result<String, String> {
    let text = datetime::ymd2_str_now(&Local);
    copy_and_notify(&app, &text, &format!("コピーしました: {text}"))
}

/// "終了(&C)": hide the tray icon implicitly (it is dropped with the app)
/// and exit for real, bypassing the CloseRequested hide-instead-of-close
/// handler installed on the main window.
#[tauri::command]
pub fn action_exit(app: AppHandle) {
    app.exit(0);
}

/// "AnnounceP": reads clipboard text, replaces newlines with `<br/>`, wraps
/// it in `<p>...</p>`, and writes the result back to the clipboard.
/// Mirrors `announcePToolStripMenuItem_Click`.
#[tauri::command]
pub fn action_announce_p(app: AppHandle) -> Result<String, String> {
    let clip = app.clipboard().read_text().map_err(|e| e.to_string())?;
    let br = clip.trim().replace("\r\n", "\n").replace('\n', "<br/>\n");
    let wrapped = format!("<p>\n{br}\n</p>");
    app.clipboard()
        .write_text(wrapped)
        .map_err(|e| e.to_string())?;
    let message = "貼付テキストを加工しました。".to_string();
    notify(&app, "TimeCopyIconRust", &message);
    Ok(message)
}

/// GroupBox1 "unixtimeを普通にする" — mirrors `textBoxFromUnixtime_Leave`.
#[tauri::command]
pub fn convert_unixtime_input(input: String) -> Option<String> {
    datetime::unixtime_input_to_local_string(&input, &Local)
}

#[derive(Serialize)]
pub struct ConversionResult {
    pub unixtime: Option<String>,
    pub message: String,
}

/// GroupBox2 "日付け文字列をunixtime" — mirrors `textBox4_Leave`.
#[tauri::command]
pub fn convert_iso8601_input(input: String) -> ConversionResult {
    if input.trim().is_empty() {
        return ConversionResult {
            unixtime: None,
            message: String::new(),
        };
    }

    match datetime::parse_flexible_datetime_to_unix(&input, &Local) {
        Some(unix) => {
            // The original re-renders the parsed instant as a local
            // date/time string for the status message, discarding any
            // offset that was present in the input.
            let local_str = datetime::unixtime_input_to_local_string(&unix.to_string(), &Local)
                .unwrap_or_default();
            ConversionResult {
                unixtime: Some(unix.to_string()),
                message: format!("変換しました。日付けは\n{local_str}"),
            }
        }
        None => ConversionResult {
            unixtime: None,
            message: "変換出来ませんでした".to_string(),
        },
    }
}

/// ISO8601 sample field focus-in — mirrors `textBoxISO8601Example_Enter`.
#[tauri::command]
pub fn copy_iso8601_example(app: AppHandle, text: String) -> Result<String, String> {
    app.clipboard()
        .write_text(text.clone())
        .map_err(|e| e.to_string())?;
    let message = format!("サンプル日付けをコピーしました。\n{text}");
    notify(&app, "TimeCopyIconRust", &message);
    Ok(message)
}

/// Initial value for the ISO8601 sample field — mirrors `Form1_Load`.
#[tauri::command]
pub fn get_iso8601_example_now() -> String {
    datetime::iso8601_example_now()
}
