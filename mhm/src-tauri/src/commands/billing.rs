use super::{emit_db_update, get_user_id, AppState};
use crate::{queries::booking::billing_queries, services::booking::billing_service};
use tauri::State;

// ═══════════════════════════════════════════════
// Phase 3: Folio / Billing Commands
// ═══════════════════════════════════════════════

#[tauri::command]
pub async fn add_folio_line(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    booking_id: String,
    category: String,
    description: String,
    amount: f64,
) -> Result<crate::models::FolioLine, String> {
    let user_id = get_user_id(&state);
    let line = billing_service::add_folio_line(
        &state.db,
        &booking_id,
        &category,
        &description,
        amount,
        user_id.as_deref(),
    )
    .await
    .map_err(|error| error.to_string())?;

    emit_db_update(&app, "folio");

    Ok(line)
}

#[tauri::command]
pub async fn get_folio_lines(
    state: State<'_, AppState>,
    booking_id: String,
) -> Result<Vec<crate::models::FolioLine>, String> {
    billing_queries::list_folio_lines(&state.db, &booking_id)
        .await
        .map_err(|e| e.to_string())
}
