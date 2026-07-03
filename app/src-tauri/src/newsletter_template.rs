use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Newsletter,
    Task,
}

impl Default for TemplateType {
    fn default() -> Self {
        Self::Newsletter
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub r#type: TemplateType,
    #[serde(default)]
    pub task_label: String,
    #[serde(default)]
    pub from_pattern: String,
    #[serde(default)]
    pub subject_pattern: String,
    #[serde(default)]
    pub prompt: String,
    /// True when loaded from the app's bundled resources (not user-created).
    #[serde(default)]
    pub builtin: bool,
}

fn user_templates_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    let dir = base.join("newsletter-templates");
    fs::create_dir_all(&dir).map_err(|e| format!("create_dir: {e}"))?;
    Ok(dir)
}

fn load_dir(dir: &std::path::Path) -> Vec<NewsletterTemplate> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .filter_map(|e| {
            let raw = fs::read_to_string(e.path()).ok()?;
            serde_json::from_str::<NewsletterTemplate>(&raw).ok()
        })
        .collect()
}

#[tauri::command]
pub fn newsletter_template_list(app: AppHandle) -> Result<Vec<NewsletterTemplate>, String> {
    // Bundled templates ship with the app inside the resources dir.
    let mut map: HashMap<String, NewsletterTemplate> = app
        .path()
        .resource_dir()
        .ok()
        .map(|r| load_dir(&r.join("newsletter-templates")))
        .unwrap_or_default()
        .into_iter()
        .map(|mut t| { t.builtin = true; (t.id.clone(), t) })
        .collect();

    // User templates override bundled ones with the same id.
    let user_dir = user_templates_dir(&app)?;
    for t in load_dir(&user_dir) {
        map.insert(t.id.clone(), t);
    }

    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn newsletter_template_save(
    app: AppHandle,
    template: NewsletterTemplate,
) -> Result<(), String> {
    if template.id.is_empty() {
        return Err("id must not be empty".into());
    }
    let dir = user_templates_dir(&app)?;
    let path = dir.join(format!("{}.json", template.id));
    let raw = serde_json::to_string_pretty(&template).map_err(|e| format!("serialize: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("write: {e}"))
}

/// Save a template into the bundled resources directory.
/// Only available in debug builds — production DMG/APK won't expose this command.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn newsletter_template_save_builtin(
    app: AppHandle,
    mut template: NewsletterTemplate,
) -> Result<(), String> {
    if template.id.is_empty() {
        return Err("id must not be empty".into());
    }
    template.builtin = true;
    let dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?
        .join("newsletter-templates");
    fs::create_dir_all(&dir).map_err(|e| format!("create_dir: {e}"))?;
    let path = dir.join(format!("{}.json", template.id));
    let raw = serde_json::to_string_pretty(&template).map_err(|e| format!("serialize: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("write: {e}"))
}

#[tauri::command]
pub fn newsletter_template_delete(app: AppHandle, id: String) -> Result<(), String> {
    let dir = user_templates_dir(&app)?;
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("delete: {e}"))?;
    }
    Ok(())
}
