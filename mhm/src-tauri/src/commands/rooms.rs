use sqlx::{Pool, Sqlite, Row};
use tauri::State;
use crate::models::*;
use super::{AppState, get_f64, emit_db_update, get_user_id};

// ─── Room Commands ───

pub async fn do_get_rooms(pool: &Pool<Sqlite>) -> Result<Vec<Room>, String> {
    let rows = sqlx::query(
        "SELECT id, name, type, floor, has_balcony, base_price, max_guests, extra_person_fee, status FROM rooms ORDER BY floor, id"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let rooms: Vec<Room> = rows.iter().map(|r| Room {
        id: r.get("id"),
        name: r.get("name"),
        room_type: r.get("type"),
        floor: r.get("floor"),
        has_balcony: r.get::<i32, _>("has_balcony") == 1,
        base_price: get_f64(r, "base_price"),
        max_guests: r.try_get::<i32, _>("max_guests").unwrap_or(2),
        extra_person_fee: r.try_get::<f64, _>("extra_person_fee").unwrap_or(0.0),
        status: r.get("status"),
    }).collect();

    Ok(rooms)
}

#[tauri::command]
pub async fn get_rooms(state: State<'_, AppState>) -> Result<Vec<Room>, String> {
    do_get_rooms(&state.db).await
}

pub async fn do_get_dashboard_stats(pool: &Pool<Sqlite>) -> Result<DashboardStats, String> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    let occupied: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms WHERE status = 'occupied'")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    let vacant: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms WHERE status = 'vacant'")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    let cleaning: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms WHERE status = 'cleaning'")
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let revenue: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(t.amount), 0) AS REAL) FROM transactions t
         WHERE t.type = 'payment' AND t.created_at LIKE ? || '%'"
    )
    .bind(&today)
    .fetch_one(pool).await.map_err(|e| e.to_string())?;

    Ok(DashboardStats {
        total_rooms: total.0 as i32,
        occupied: occupied.0 as i32,
        vacant: vacant.0 as i32,
        cleaning: cleaning.0 as i32,
        revenue_today: revenue.0,
    })
}

#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    do_get_dashboard_stats(&state.db).await
}

// ─── Check-in Command ───

#[tauri::command]
pub async fn check_in(state: State<'_, AppState>, app: tauri::AppHandle, req: CheckInRequest) -> Result<Booking, String> {
    // Input validation
    if req.guests.is_empty() {
        return Err("Phải có ít nhất 1 khách".to_string());
    }
    if req.nights <= 0 {
        return Err("Number of nights must be greater than 0".to_string());
    }

    let user_id = get_user_id(&state);

    // Start transaction — all DB operations are atomic
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    let room_status: (String,) = sqlx::query_as("SELECT status FROM rooms WHERE id = ?")
        .bind(&req.room_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

    if room_status.0 != "vacant" {
        return Err(format!("Phòng {} không trống (status: {})", req.room_id, room_status.0));
    }

    // ── Calendar overlap check (overbooking prevention) ──
    let now = chrono::Local::now();
    let checkin_date = now.format("%Y-%m-%d").to_string();
    let checkout = now + chrono::Duration::days(req.nights as i64);
    let checkout_date = checkout.format("%Y-%m-%d").to_string();

    let conflicts = sqlx::query(
        "SELECT rc.date, rc.status, rc.booking_id, COALESCE(g.full_name, '') as guest_name
         FROM room_calendar rc
         LEFT JOIN bookings b ON b.id = rc.booking_id
         LEFT JOIN guests g ON g.id = b.primary_guest_id
         WHERE rc.room_id = ? AND rc.date >= ? AND rc.date < ?
         ORDER BY rc.date ASC"
    )
    .bind(&req.room_id).bind(&checkin_date).bind(&checkout_date)
    .fetch_all(&mut *tx).await.map_err(|e| e.to_string())?;

    if !conflicts.is_empty() {
        let first_date: String = conflicts[0].get("date");
        let guest: String = conflicts[0].get("guest_name");
        // Calculate max nights until first conflict
        let first_conflict = chrono::NaiveDate::parse_from_str(&first_date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?;
        let today = now.date_naive();
        let max_nights = (first_conflict - today).num_days();
        return Err(format!(
            "Room {} has a reservation starting {} ({}). Max {} nights.",
            req.room_id, first_date, guest, max_nights
        ));
    }

    let price_row = sqlx::query("SELECT base_price FROM rooms WHERE id = ?")
        .bind(&req.room_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    let room_price = get_f64(&price_row, "base_price");

    let total_price = room_price * req.nights as f64;
    let paid = req.paid_amount.unwrap_or(0.0);

    // Create primary guest
    let primary_guest_id = uuid::Uuid::new_v4().to_string();
    let primary = &req.guests[0];

    sqlx::query(
        "INSERT INTO guests (id, guest_type, full_name, doc_number, dob, gender, nationality, address, visa_expiry, scan_path, phone, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&primary_guest_id)
    .bind(primary.guest_type.as_deref().unwrap_or("domestic"))
    .bind(&primary.full_name)
    .bind(&primary.doc_number)
    .bind(&primary.dob)
    .bind(&primary.gender)
    .bind(&primary.nationality)
    .bind(&primary.address)
    .bind(&primary.visa_expiry)
    .bind(&primary.scan_path)
    .bind(&primary.phone)
    .bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Create booking
    let booking_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bookings (id, room_id, primary_guest_id, check_in_at, expected_checkout, nights, total_price, paid_amount, status, source, notes, created_by, booking_type, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?, 'walk-in', ?)"
    )
    .bind(&booking_id)
    .bind(&req.room_id)
    .bind(&primary_guest_id)
    .bind(now.to_rfc3339())
    .bind(checkout.to_rfc3339())
    .bind(req.nights)
    .bind(total_price)
    .bind(paid)
    .bind(req.source.as_deref().unwrap_or("walk-in"))
    .bind(&req.notes)
    .bind(&user_id)
    .bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Link primary guest
    sqlx::query("INSERT INTO booking_guests (booking_id, guest_id) VALUES (?, ?)")
        .bind(&booking_id).bind(&primary_guest_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Add additional guests
    for guest_req in req.guests.iter().skip(1) {
        let guest_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO guests (id, guest_type, full_name, doc_number, dob, gender, nationality, address, visa_expiry, scan_path, phone, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&guest_id)
        .bind(guest_req.guest_type.as_deref().unwrap_or("domestic"))
        .bind(&guest_req.full_name)
        .bind(&guest_req.doc_number)
        .bind(&guest_req.dob)
        .bind(&guest_req.gender)
        .bind(&guest_req.nationality)
        .bind(&guest_req.address)
        .bind(&guest_req.visa_expiry)
        .bind(&guest_req.scan_path)
        .bind(&guest_req.phone)
        .bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        sqlx::query("INSERT INTO booking_guests (booking_id, guest_id) VALUES (?, ?)")
            .bind(&booking_id).bind(&guest_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // Always record charge transaction for room revenue
    let charge_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
         VALUES (?, ?, ?, 'charge', 'Tiền phòng', ?)"
    )
    .bind(&charge_id).bind(&booking_id).bind(total_price).bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Record payment if any
    if paid > 0.0 {
        let txn_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
             VALUES (?, ?, ?, 'payment', 'Thanh toán khi check-in', ?)"
        )
        .bind(&txn_id).bind(&booking_id).bind(paid).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // Insert calendar rows for each day of the stay (occupied)
    let mut d = now.date_naive();
    let checkout_naive = checkout.date_naive();
    while d < checkout_naive {
        sqlx::query(
            "INSERT OR REPLACE INTO room_calendar (room_id, date, booking_id, status) VALUES (?, ?, ?, 'occupied')"
        )
        .bind(&req.room_id).bind(d.format("%Y-%m-%d").to_string()).bind(&booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        d += chrono::Duration::days(1);
    }

    // Update room status
    sqlx::query("UPDATE rooms SET status = 'occupied' WHERE id = ?")
        .bind(&req.room_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Commit transaction — all-or-nothing
    tx.commit().await.map_err(|e| e.to_string())?;

    emit_db_update(&app, "rooms");

    Ok(Booking {
        id: booking_id,
        room_id: req.room_id,
        primary_guest_id,
        check_in_at: now.to_rfc3339(),
        expected_checkout: checkout.to_rfc3339(),
        actual_checkout: None,
        nights: req.nights,
        total_price,
        paid_amount: paid,
        status: "active".to_string(),
        source: req.source,
        notes: req.notes,
        created_at: now.to_rfc3339(),
    })
}

// ─── Room Detail Command ───

pub async fn do_get_room_detail(pool: &Pool<Sqlite>, room_id: &str) -> Result<RoomWithBooking, String> {
    let row = sqlx::query("SELECT id, name, type, floor, has_balcony, base_price, max_guests, extra_person_fee, status FROM rooms WHERE id = ?")
        .bind(room_id)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    let room = Room {
        id: row.get("id"),
        name: row.get("name"),
        room_type: row.get("type"),
        floor: row.get("floor"),
        has_balcony: row.get::<i32, _>("has_balcony") == 1,
        base_price: get_f64(&row, "base_price"),
        max_guests: row.try_get::<i32, _>("max_guests").unwrap_or(2),
        extra_person_fee: row.try_get::<f64, _>("extra_person_fee").unwrap_or(0.0),
        status: row.get("status"),
    };

    let booking = sqlx::query(
        "SELECT id, room_id, primary_guest_id, check_in_at, expected_checkout, actual_checkout, nights, total_price, paid_amount, status, source, notes, created_at
         FROM bookings WHERE room_id = ? AND status = 'active' LIMIT 1"
    )
    .bind(room_id)
    .fetch_optional(pool).await.map_err(|e| e.to_string())?
    .map(|r| Booking {
        id: r.get("id"),
        room_id: r.get("room_id"),
        primary_guest_id: r.get("primary_guest_id"),
        check_in_at: r.get("check_in_at"),
        expected_checkout: r.get("expected_checkout"),
        actual_checkout: r.get("actual_checkout"),
        nights: r.get("nights"),
        total_price: get_f64(&r, "total_price"),
        paid_amount: get_f64(&r, "paid_amount"),
        status: r.get("status"),
        source: r.get("source"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
    });

    let guests = if let Some(ref b) = booking {
        let rows = sqlx::query(
            "SELECT g.* FROM guests g
             JOIN booking_guests bg ON bg.guest_id = g.id
             WHERE bg.booking_id = ?"
        )
        .bind(&b.id)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        rows.iter().map(|r| Guest {
            id: r.get("id"),
            guest_type: r.get("guest_type"),
            full_name: r.get("full_name"),
            doc_number: r.get("doc_number"),
            dob: r.get("dob"),
            gender: r.get("gender"),
            nationality: r.get("nationality"),
            address: r.get("address"),
            visa_expiry: r.get("visa_expiry"),
            scan_path: r.get("scan_path"),
            phone: r.get("phone"),
            created_at: r.get("created_at"),
        }).collect()
    } else {
        vec![]
    };

    Ok(RoomWithBooking { room, booking, guests })
}

#[tauri::command]
pub async fn get_room_detail(state: State<'_, AppState>, room_id: String) -> Result<RoomWithBooking, String> {
    do_get_room_detail(&state.db, &room_id).await
}

// ─── Check-out Command ───

#[tauri::command]
pub async fn check_out(state: State<'_, AppState>, app: tauri::AppHandle, req: CheckOutRequest) -> Result<(), String> {
    let now = chrono::Local::now();

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // Get booking
    let row = sqlx::query("SELECT room_id, total_price, paid_amount FROM bookings WHERE id = ? AND status = 'active'")
        .bind(&req.booking_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

    let room_id: String = row.get("room_id");
    let _total: f64 = get_f64(&row, "total_price");
    let already_paid: f64 = get_f64(&row, "paid_amount");

    // Record final payment if provided
    if let Some(final_paid) = req.final_paid {
        let additional = final_paid - already_paid;
        if additional > 0.0 {
            let txn_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
                 VALUES (?, ?, ?, 'payment', 'Thanh toán khi check-out', ?)"
            )
            .bind(&txn_id).bind(&req.booking_id).bind(additional).bind(now.to_rfc3339())
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

            sqlx::query("UPDATE bookings SET paid_amount = ? WHERE id = ?")
                .bind(final_paid).bind(&req.booking_id)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
    } else {
        // No final_paid provided — keep existing paid_amount as-is (allow outstanding balance/debt)
    }

    // Update booking
    sqlx::query("UPDATE bookings SET status = 'checked_out', actual_checkout = ? WHERE id = ?")
        .bind(now.to_rfc3339()).bind(&req.booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Room → cleaning
    sqlx::query("UPDATE rooms SET status = 'cleaning' WHERE id = ?")
        .bind(&room_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Create housekeeping task
    let hk_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO housekeeping (id, room_id, status, triggered_at, created_at)
         VALUES (?, ?, 'needs_cleaning', ?, ?)"
    )
    .bind(&hk_id).bind(&room_id).bind(now.to_rfc3339()).bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Clean up calendar rows for this booking
    sqlx::query("DELETE FROM room_calendar WHERE booking_id = ?")
        .bind(&req.booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Commit transaction
    tx.commit().await.map_err(|e| e.to_string())?;

    emit_db_update(&app, "rooms");

    Ok(())
}

// ─── Extend Stay ───

#[tauri::command]
pub async fn extend_stay(state: State<'_, AppState>, booking_id: String) -> Result<Booking, String> {
    let row = sqlx::query("SELECT * FROM bookings WHERE id = ? AND status = 'active'")
        .bind(&booking_id)
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let room_id: String = row.get("room_id");
    let nights: i32 = row.get("nights");
    let old_total: f64 = get_f64(&row, "total_price");
    let expected_co: String = row.get("expected_checkout");

    let price_row = sqlx::query("SELECT base_price FROM rooms WHERE id = ?")
        .bind(&room_id)
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;
    let room_price = get_f64(&price_row, "base_price");

    let new_nights = nights + 1;
    let new_total = old_total + room_price;

    // Parse existing expected_checkout and add 1 day (not from now)
    let co_dt = chrono::DateTime::parse_from_rfc3339(&expected_co)
        .map(|d| d.with_timezone(&chrono::Local))
        .map_err(|e| format!("Cannot parse expected_checkout: {}", e))?;
    let new_checkout = co_dt + chrono::Duration::days(1);

    sqlx::query("UPDATE bookings SET nights = ?, total_price = ?, expected_checkout = ? WHERE id = ?")
        .bind(new_nights).bind(new_total).bind(new_checkout.to_rfc3339()).bind(&booking_id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    // Record charge for the additional night
    let charge_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now();
    sqlx::query(
        "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
         VALUES (?, ?, ?, 'charge', 'Extended stay +1 night', ?)"
    )
    .bind(&charge_id).bind(&booking_id).bind(room_price).bind(now.to_rfc3339())
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    get_booking_by_id(&state.db, &booking_id).await
}

pub(crate) async fn get_booking_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Booking, String> {
    let r = sqlx::query("SELECT * FROM bookings WHERE id = ?")
        .bind(id)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    Ok(Booking {
        id: r.get("id"),
        room_id: r.get("room_id"),
        primary_guest_id: r.get("primary_guest_id"),
        check_in_at: r.get("check_in_at"),
        expected_checkout: r.get("expected_checkout"),
        actual_checkout: r.get("actual_checkout"),
        nights: r.get("nights"),
        total_price: get_f64(&r, "total_price"),
        paid_amount: get_f64(&r, "paid_amount"),
        status: r.get("status"),
        source: r.get("source"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
    })
}

// ─── Housekeeping Commands ───

#[tauri::command]
pub async fn get_housekeeping_tasks(state: State<'_, AppState>) -> Result<Vec<HousekeepingTask>, String> {
    let rows = sqlx::query("SELECT * FROM housekeeping WHERE status != 'clean' ORDER BY triggered_at ASC")
        .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| HousekeepingTask {
        id: r.get("id"),
        room_id: r.get("room_id"),
        status: r.get("status"),
        note: r.get("note"),
        triggered_at: r.get("triggered_at"),
        cleaned_at: r.get("cleaned_at"),
        created_at: r.get("created_at"),
    }).collect())
}

#[tauri::command]
pub async fn update_housekeeping(state: State<'_, AppState>, app: tauri::AppHandle, task_id: String, new_status: String, note: Option<String>) -> Result<(), String> {
    let now = chrono::Local::now();

    let cleaned_at = if new_status == "clean" { Some(now.to_rfc3339()) } else { None };

    sqlx::query("UPDATE housekeeping SET status = ?, note = COALESCE(?, note), cleaned_at = ? WHERE id = ?")
        .bind(&new_status).bind(&note).bind(&cleaned_at).bind(&task_id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    // If clean, update room to vacant
    if new_status == "clean" {
        let room_id: (String,) = sqlx::query_as("SELECT room_id FROM housekeeping WHERE id = ?")
            .bind(&task_id)
            .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

        sqlx::query("UPDATE rooms SET status = 'vacant' WHERE id = ?")
            .bind(&room_id.0)
            .execute(&state.db).await.map_err(|e| e.to_string())?;
    }

    emit_db_update(&app, "housekeeping");

    Ok(())
}

// ─── Expense Commands ───

#[tauri::command]
pub async fn create_expense(state: State<'_, AppState>, req: CreateExpenseRequest) -> Result<Expense, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO expenses (id, category, amount, note, expense_date, created_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&req.category).bind(req.amount).bind(&req.note).bind(&req.expense_date).bind(&now)
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    Ok(Expense {
        id,
        category: req.category,
        amount: req.amount,
        note: req.note,
        expense_date: req.expense_date,
        created_at: now,
    })
}

#[tauri::command]
pub async fn get_expenses(state: State<'_, AppState>, from: String, to: String) -> Result<Vec<Expense>, String> {
    let rows = sqlx::query(
        "SELECT * FROM expenses WHERE expense_date BETWEEN ? AND ? ORDER BY expense_date DESC"
    )
    .bind(&from).bind(&to)
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| Expense {
        id: r.get("id"),
        category: r.get("category"),
        amount: get_f64(r, "amount"),
        note: r.get("note"),
        expense_date: r.get("expense_date"),
        created_at: r.get("created_at"),
    }).collect())
}

// ─── Statistics Commands ───

#[tauri::command]
pub async fn get_revenue_stats(state: State<'_, AppState>, from: String, to: String) -> Result<RevenueStats, String> {
    let total: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM transactions WHERE type = 'payment' AND created_at BETWEEN ? AND ?"
    )
    .bind(&from).bind(&to)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let rooms_sold: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT room_id) FROM bookings WHERE check_in_at BETWEEN ? AND ?"
    )
    .bind(&from).bind(&to)
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let daily_rows = sqlx::query(
        "SELECT DATE(created_at) as d, SUM(amount) as rev FROM transactions
         WHERE type = 'payment' AND created_at BETWEEN ? AND ?
         GROUP BY DATE(created_at) ORDER BY d"
    )
    .bind(&from).bind(&to)
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let daily_revenue: Vec<DailyRevenue> = daily_rows.iter().map(|r| DailyRevenue {
        date: r.get("d"),
        revenue: get_f64(r, "rev"),
    }).collect();

    let total_rooms: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms")
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    Ok(RevenueStats {
        total_revenue: total.0,
        rooms_sold: rooms_sold.0 as i32,
        occupancy_rate: if total_rooms.0 > 0 { (rooms_sold.0 as f64 / total_rooms.0 as f64) * 100.0 } else { 0.0 },
        daily_revenue,
    })
}

// ─── Copy Lưu Trú ───

#[tauri::command]
pub async fn get_stay_info_text(state: State<'_, AppState>, booking_id: String) -> Result<String, String> {
    let b = sqlx::query("SELECT * FROM bookings WHERE id = ?")
        .bind(&booking_id).fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let g = sqlx::query("SELECT * FROM guests WHERE id = ?")
        .bind(b.get::<String, _>("primary_guest_id"))
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let room_id: String = b.get("room_id");
    let full_name: String = g.get("full_name");
    let doc_number: String = g.get("doc_number");
    let dob: String = g.get::<Option<String>, _>("dob").unwrap_or_default();
    let gender: String = g.get::<Option<String>, _>("gender").unwrap_or_default();
    let nationality: String = g.get::<Option<String>, _>("nationality").unwrap_or_else(|| "Việt Nam".to_string());
    let address: String = g.get::<Option<String>, _>("address").unwrap_or_default();
    let check_in: String = b.get("check_in_at");
    let checkout: String = b.get("expected_checkout");

    let text = format!(
        "Họ và tên: {}\nSố CCCD: {}\nNgày sinh: {}\nGiới tính: {}\nQuốc tịch: {}\nĐịa chỉ: {}\nPhòng: {}\nNgày đến: {}\nNgày đi: {}",
        full_name, doc_number, dob, gender, nationality, address, room_id, check_in, checkout
    );

    Ok(text)
}

// ─── OCR Scan Command ───

#[tauri::command]
pub async fn scan_image(path: String) -> Result<crate::ocr::CccdInfo, String> {
    let image_path = std::path::Path::new(&path);
    if !image_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    let engine = crate::ocr::create_engine()?;
    let lines = crate::ocr::ocr_image(&engine, image_path)?;
    let cccd = crate::ocr::parse_cccd(&lines);

    Ok(cccd)
}
