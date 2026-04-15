use sqlx::Row;
use tauri::State;
use super::{AppState, get_f64, emit_db_update, get_user_id, require_admin};


// ═══════════════════════════════════════════════
// Phase 4: Night Audit Commands
// ═══════════════════════════════════════════════

#[tauri::command]
pub async fn run_night_audit(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    audit_date: String,
    notes: Option<String>,
) -> Result<serde_json::Value, String> {
    require_admin(&state)?;
    let user_id = get_user_id(&state);

    // Check if already audited
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM night_audit_logs WHERE audit_date = ?"
    ).bind(&audit_date).fetch_optional(&state.db).await.map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Err(format!("Đã audit ngày {} rồi!", audit_date));
    }

    // Calculate room revenue for the date
    let room_rev: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(total_price * 1.0 / nights), 0) AS REAL) FROM bookings
         WHERE DATE(check_in_at) <= ? AND DATE(expected_checkout) >= ?
         AND status IN ('active', 'checked_out')"
    ).bind(&audit_date).bind(&audit_date)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    // Calculate folio revenue for the date
    let folio_rev: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(fl.amount), 0) AS REAL) FROM folio_lines fl
         JOIN bookings b ON fl.booking_id = b.id
         WHERE DATE(fl.created_at) = ?"
    ).bind(&audit_date)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    // Calculate expenses for the date
    let expenses: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM expenses WHERE expense_date = ?"
    ).bind(&audit_date)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    // Calculate occupancy
    let total_rooms: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM rooms")
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let rooms_sold: (i32,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT room_id) FROM bookings
         WHERE DATE(check_in_at) <= ? AND DATE(expected_checkout) >= ?
         AND status IN ('active', 'checked_out')"
    ).bind(&audit_date).bind(&audit_date)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let occupancy = if total_rooms.0 > 0 {
        (rooms_sold.0 as f64 / total_rooms.0 as f64 * 100.0).round()
    } else { 0.0 };

    let total_revenue = room_rev.0 + folio_rev.0;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO night_audit_logs
         (id, audit_date, total_revenue, room_revenue, folio_revenue,
          total_expenses, occupancy_pct, rooms_sold, total_rooms,
          notes, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&audit_date).bind(total_revenue).bind(room_rev.0)
    .bind(folio_rev.0).bind(expenses.0).bind(occupancy)
    .bind(rooms_sold.0).bind(total_rooms.0)
    .bind(&notes).bind(&user_id).bind(&now)
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    // Mark bookings as audited
    sqlx::query(
        "UPDATE bookings SET is_audited = 1
         WHERE DATE(check_in_at) <= ? AND status IN ('active', 'checked_out')"
    ).bind(&audit_date)
    .execute(&state.db).await.ok();

    emit_db_update(&app, "audit");

    Ok(serde_json::json!({
        "id": id,
        "audit_date": audit_date,
        "total_revenue": total_revenue,
        "room_revenue": room_rev.0,
        "folio_revenue": folio_rev.0,
        "total_expenses": expenses.0,
        "occupancy_pct": occupancy,
        "rooms_sold": rooms_sold.0,
        "total_rooms": total_rooms.0,
    }))
}

#[tauri::command]
pub async fn get_audit_logs(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query(
        "SELECT id, audit_date, total_revenue, room_revenue, folio_revenue,
                total_expenses, occupancy_pct, rooms_sold, total_rooms, notes, created_at
         FROM night_audit_logs ORDER BY audit_date DESC LIMIT 30"
    ).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| serde_json::json!({
        "id": r.get::<String, _>("id"),
        "audit_date": r.get::<String, _>("audit_date"),
        "total_revenue": get_f64(r, "total_revenue"),
        "room_revenue": get_f64(r, "room_revenue"),
        "folio_revenue": get_f64(r, "folio_revenue"),
        "total_expenses": get_f64(r, "total_expenses"),
        "occupancy_pct": get_f64(r, "occupancy_pct"),
        "rooms_sold": r.get::<i32, _>("rooms_sold"),
        "total_rooms": r.get::<i32, _>("total_rooms"),
        "notes": r.get::<Option<String>, _>("notes"),
        "created_at": r.get::<String, _>("created_at"),
    })).collect())
}

// ═══════════════════════════════════════════════
// Phase 5: Backup & Data Export
// ═══════════════════════════════════════════════

#[tauri::command]
pub async fn backup_database() -> Result<String, String> {
    let db_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join("MHM");

    let db_path = db_dir.join("mhm.db");
    if !db_path.exists() {
        return Err("Database file not found".to_string());
    }

    let backup_dir = db_dir.join("backups");
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_dir.join(format!("mhm_backup_{}.db", timestamp));

    std::fs::copy(&db_path, &backup_path).map_err(|e| e.to_string())?;

    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn export_bookings_csv(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<String, String> {
    let from = from_date.unwrap_or_else(|| "2000-01-01".to_string());
    let to = to_date.unwrap_or_else(|| "2099-12-31".to_string());

    let rows = sqlx::query(
        "SELECT b.id, b.room_id, g.full_name, g.doc_number, g.phone,
                b.check_in_at, b.expected_checkout, b.actual_checkout,
                b.nights, b.total_price, b.paid_amount, b.status,
                b.pricing_type, b.source,
                COALESCE((SELECT SUM(fl.amount) FROM folio_lines fl WHERE fl.booking_id = b.id), 0) as folio_total
         FROM bookings b
         LEFT JOIN guests g ON b.primary_guest_id = g.id
         WHERE DATE(b.check_in_at) >= ? AND DATE(b.check_in_at) <= ?
         ORDER BY b.check_in_at DESC"
    )
    .bind(&from).bind(&to)
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let mut csv = String::from("ID,Room,Guest,DocNumber,Phone,CheckIn,CheckOut,ActualCheckout,Nights,RoomPrice,FolioTotal,PaidAmount,Status,PricingType,Source\n");

    for r in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.get::<String, _>("id"),
            r.get::<String, _>("room_id"),
            r.get::<String, _>("full_name").replace(',', " "),
            r.get::<String, _>("doc_number"),
            r.get::<Option<String>, _>("phone").unwrap_or_default(),
            r.get::<String, _>("check_in_at"),
            r.get::<String, _>("expected_checkout"),
            r.get::<Option<String>, _>("actual_checkout").unwrap_or_default(),
            r.get::<i32, _>("nights"),
            get_f64(r, "total_price"),
            get_f64(r, "folio_total"),
            get_f64(r, "paid_amount"),
            r.get::<String, _>("status"),
            r.get::<Option<String>, _>("pricing_type").unwrap_or_default(),
            r.get::<Option<String>, _>("source").unwrap_or_default(),
        ));
    }

    // Save to file
    let export_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join("MHM").join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_path = export_dir.join(format!("bookings_{}.csv", timestamp));

    std::fs::write(&file_path, &csv).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}
