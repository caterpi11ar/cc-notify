use tauri::State;
use crate::store::AppState;
use crate::models::Rule;

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>) -> Result<Vec<Rule>, String> {
    state.db.get_all_rules().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_rule(state: State<'_, AppState>, rule: Rule) -> Result<Rule, String> {
    let mut r = rule;
    if r.id.is_empty() {
        r.id = uuid::Uuid::new_v4().to_string();
    }
    if r.created_at == 0 {
        r.created_at = chrono::Utc::now().timestamp();
    }
    state.db.insert_rule(&r).map_err(|e| e.to_string())?;
    Ok(r)
}

#[tauri::command]
pub fn update_rule(
    state: State<'_, AppState>,
    id: String,
    rule: serde_json::Value,
) -> Result<(), String> {
    let existing = state
        .db
        .get_rule(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Rule not found: {id}"))?;

    let mut merged = serde_json::to_value(&existing).map_err(|e| e.to_string())?;
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), rule.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }

    let updated: Rule = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    state.db.update_rule(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_rule(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_rule(&id).map_err(|e| e.to_string())?;
    Ok(())
}
