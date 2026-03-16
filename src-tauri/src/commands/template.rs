use tauri::State;
use crate::store::AppState;
use crate::models::Template;

#[tauri::command]
pub fn get_templates(state: State<'_, AppState>) -> Result<Vec<Template>, String> {
    state.db.get_all_templates().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_template(
    state: State<'_, AppState>,
    template: Template,
) -> Result<Template, String> {
    let mut t = template;
    if t.id.is_empty() {
        t.id = uuid::Uuid::new_v4().to_string();
    }
    state.db.insert_template(&t).map_err(|e| e.to_string())?;
    Ok(t)
}

#[tauri::command]
pub fn update_template(
    state: State<'_, AppState>,
    id: String,
    template: serde_json::Value,
) -> Result<(), String> {
    let existing = state
        .db
        .get_template(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Template not found: {id}"))?;

    let mut merged = serde_json::to_value(&existing).map_err(|e| e.to_string())?;
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), template.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }

    let updated: Template = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    state
        .db
        .update_template(&updated)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_template(&id).map_err(|e| e.to_string())?;
    Ok(())
}
