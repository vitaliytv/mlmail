//! Tauri application shell that wires authentication, Gmail, plugins and LLM commands.

/// OAuth authentication commands and persistent state.
pub mod auth;
#[cfg(debug_assertions)]
/// Debug-only cloud analysis command.
pub mod call_analysis;
/// Shared remote endpoint definitions.
pub mod endpoints;
/// Gmail message and filter commands.
pub mod gmail;
/// OpenAI-compatible local LLM commands.
pub mod llm;
/// Newsletter template persistence commands.
pub mod newsletter_template;
/// Durable lifecycle coordinator for product plugin instances.
pub mod plugin_context;
/// Product-owned typed plugin contract registry.
pub mod plugin_contracts;
/// Exact installed Component selection for typed plugin commands.
pub mod plugin_dispatch;
/// Product-owned exact plugin capability grants.
pub mod plugin_grants;
/// Pure installation preflight for typed plugin Components.
pub mod plugin_install;
/// Component-only installed plugin commands.
pub mod plugins;

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
/// Starts the Tauri runtime and registers the application's command surface.
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_agent::init());

    #[cfg(desktop)]
    let builder = builder.plugin(
        tauri_plugin_window_state::Builder::default()
            .with_state_flags(
                tauri_plugin_window_state::StateFlags::all()
                    & !tauri_plugin_window_state::StateFlags::MAXIMIZED,
            )
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
        .manage(plugins::PluginInstallState::default())
        .manage(endpoints::Endpoints::default())
        .setup(|app| {
            let handle = app.handle();
            let storage = auth::make_storage(handle);
            auth::on_startup(handle, storage.as_ref())?;
            app.manage(storage);
            plugins::reconcile_pending_installs(handle)?;
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
            // jscpd:ignore-start -- the two generate_handler! lists are unavoidably
            // near-identical: `call_analysis` (and its one command) only exists
            // under #[cfg(debug_assertions)], so the release list can't reuse it.
            #[cfg(not(debug_assertions))]
            {
                tauri::generate_handler![
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
                    gmail::gmail_open_attachment,
                    gmail::gmail_trash,
                    gmail::gmail_trash_query,
                    gmail::gmail_save,
                    gmail::gmail_flag_task,
                    gmail::gmail_unflag_task,
                    gmail::gmail_create_filter,
                    gmail::gmail_list_filters,
                    gmail::gmail_delete_filter,
                    gmail::gmail_list_labels,
                    newsletter_template::newsletter_template_list,
                    newsletter_template::newsletter_template_save,
                    newsletter_template::newsletter_template_delete,
                    llm::llm_default_config,
                    llm::llm_providers,
                    llm::llm_list_models,
                    llm::llm_chat,
                    plugins::plugin_manager_list,
                    plugins::plugin_manager_preflight,
                    plugins::plugin_manager_confirm_install,
                    plugins::plugin_manager_install,
                    plugins::plugin_manager_set_disabled,
                    plugins::plugin_manager_uninstall,
                    plugins::plugin_draft_helper_create,
                    plugins::plugin_booking_finder_find,
                    app_open_url,
                    app_set_title,
                ]
            }
            #[cfg(debug_assertions)]
            {
                tauri::generate_handler![
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
                    gmail::gmail_open_attachment,
                    gmail::gmail_trash,
                    gmail::gmail_trash_query,
                    gmail::gmail_save,
                    gmail::gmail_flag_task,
                    gmail::gmail_unflag_task,
                    gmail::gmail_create_filter,
                    gmail::gmail_list_filters,
                    gmail::gmail_delete_filter,
                    gmail::gmail_list_labels,
                    newsletter_template::newsletter_template_list,
                    newsletter_template::newsletter_template_save,
                    newsletter_template::newsletter_template_delete,
                    newsletter_template::newsletter_template_save_builtin,
                    call_analysis::analyze_call_with_pi,
                    llm::llm_default_config,
                    llm::llm_providers,
                    llm::llm_list_models,
                    llm::llm_chat,
                    plugins::plugin_manager_list,
                    plugins::plugin_manager_preflight,
                    plugins::plugin_manager_confirm_install,
                    plugins::plugin_manager_install,
                    plugins::plugin_manager_set_disabled,
                    plugins::plugin_manager_uninstall,
                    plugins::plugin_draft_helper_create,
                    plugins::plugin_booking_finder_find,
                    app_open_url,
                    app_set_title,
                ]
            }
            // jscpd:ignore-end
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
