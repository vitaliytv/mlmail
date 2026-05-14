pub mod auth;
pub mod gmail;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(auth::state::AuthState::default()))
        .setup(|app| {
            auth::on_startup(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::auth_start_login,
            auth::auth_get_access_token,
            auth::auth_is_authenticated,
            auth::auth_current_email,
            auth::auth_logout,
            gmail::gmail_inbox_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
