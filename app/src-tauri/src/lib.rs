pub mod auth;
pub mod endpoints;
pub mod gmail;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(auth::state::AuthState::default()))
        .manage(endpoints::Endpoints::default())
        .setup(|app| {
            let handle = app.handle();
            let storage = auth::make_storage(handle);
            auth::on_startup(handle, storage.as_ref())?;
            app.manage(storage);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::auth_start_login,
            auth::auth_get_access_token,
            auth::auth_is_authenticated,
            auth::auth_current_email,
            auth::auth_logout,
            gmail::gmail_inbox_count,
            gmail::gmail_random_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
