pub mod auth;
pub mod endpoints;
pub mod gmail;
pub mod newsletter_template;

use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
fn app_open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn app_set_title(app: tauri::AppHandle, title: String) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title(&title);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_agent::init());

    #[cfg(desktop)]
    let builder = builder.plugin(
        tauri_plugin_window_state::Builder::default()
            .with_state_flags(tauri_plugin_window_state::StateFlags::all() & !tauri_plugin_window_state::StateFlags::MAXIMIZED)
            .build(),
    );

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    // relaunch() після встановлення оновлення — щоб застосунок сам
    // перезапустився в нову версію, а не чекав ручного рестарту.
    let builder = builder.plugin(tauri_plugin_process::init());

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    builder
        .manage(Mutex::new(auth::state::AuthState::default()))
        .manage(endpoints::Endpoints::default())
        .setup(|app| {
            let handle = app.handle();
            let storage = auth::make_storage(handle);
            auth::on_startup(handle, storage.as_ref())?;
            app.manage(storage);
            // Maximize after window-state plugin finishes restoring geometry.
            #[cfg(desktop)]
            {
                let handle2 = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    if let Some(w) = handle2.get_webview_window("main") {
                        let _ = w.maximize();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler({
            #[cfg(not(debug_assertions))]
            { tauri::generate_handler![
                auth::auth_start_login,
                auth::auth_get_access_token,
                auth::auth_is_authenticated,
                auth::auth_current_email,
                auth::auth_logout,
                gmail::gmail_inbox_count,
                gmail::gmail_random_message,
                gmail::gmail_random_newsletter,
                gmail::gmail_unsubscribe,
                gmail::gmail_search,
                gmail::gmail_read,
                gmail::gmail_trash,
                gmail::gmail_trash_query,
                gmail::gmail_save,
                gmail::gmail_create_filter,
                newsletter_template::newsletter_template_list,
                newsletter_template::newsletter_template_save,
                newsletter_template::newsletter_template_delete,
                app_open_url,
                app_set_title,
            ] }
            #[cfg(debug_assertions)]
            { tauri::generate_handler![
                auth::auth_start_login,
                auth::auth_get_access_token,
                auth::auth_is_authenticated,
                auth::auth_current_email,
                auth::auth_logout,
                gmail::gmail_inbox_count,
                gmail::gmail_random_message,
                gmail::gmail_random_newsletter,
                gmail::gmail_unsubscribe,
                gmail::gmail_search,
                gmail::gmail_read,
                gmail::gmail_trash,
                gmail::gmail_trash_query,
                gmail::gmail_save,
                gmail::gmail_create_filter,
                newsletter_template::newsletter_template_list,
                newsletter_template::newsletter_template_save,
                newsletter_template::newsletter_template_delete,
                newsletter_template::newsletter_template_save_builtin,
                app_open_url,
                app_set_title,
            ] }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
