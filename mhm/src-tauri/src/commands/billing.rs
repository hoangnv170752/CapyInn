use sqlx::Row;
use tauri::State;
use super::{AppState, get_f64, emit_db_update, get_user_id};


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
) -> Result<serde_json::Value, String> {
    let user_id = get_user_id(&state);
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO folio_lines (id, booking_id, category, description, amount, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&booking_id).bind(&category).bind(&description)
    .bind(amount).bind(&user_id).bind(&now)
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    emit_db_update(&app, "folio");

    Ok(serde_json::json!({
        "id": id,
        "booking_id": booking_id,
        "category": category,
        "description": description,
        "amount": amount,
        "created_at": now,
    }))
}

#[tauri::command]
pub async fn get_folio_lines(
    state: State<'_, AppState>,
    booking_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query(
        "SELECT id, booking_id, category, description, amount, created_by, created_at
         FROM folio_lines WHERE booking_id = ? ORDER BY created_at"
    ).bind(&booking_id).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| serde_json::json!({
        "id": r.get::<String, _>("id"),
        "booking_id": r.get::<String, _>("booking_id"),
        "category": r.get::<String, _>("category"),
        "description": r.get::<String, _>("description"),
        "amount": get_f64(r, "amount"),
        "created_at": r.get::<String, _>("created_at"),
    })).collect())
}
