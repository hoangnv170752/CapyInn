use super::{emit_db_update, get_f64, AppState};
use crate::models::*;
use crate::services::booking::reservation_lifecycle;
use sqlx::{Pool, Row, Sqlite};
use tauri::State;

// ═══════════════════════════════════════════════
// Reservation Calendar Block System
// ═══════════════════════════════════════════════

// ─── Check Availability ───

pub async fn do_check_availability(pool: &Pool<Sqlite>, room_id: &str, from_date: &str, to_date: &str) -> Result<AvailabilityResult, String> {
    let rows = sqlx::query(
        "SELECT rc.date, rc.status, rc.booking_id, COALESCE(g.full_name, '') as guest_name
         FROM room_calendar rc
         LEFT JOIN bookings b ON b.id = rc.booking_id
         LEFT JOIN guests g ON g.id = b.primary_guest_id
         WHERE rc.room_id = ? AND rc.date >= ? AND rc.date < ?
         ORDER BY rc.date ASC"
    )
    .bind(room_id).bind(from_date).bind(to_date)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(AvailabilityResult {
            available: true,
            conflicts: vec![],
            max_nights: None,
        });
    }

    let conflicts: Vec<CalendarConflict> = rows.iter().map(|r| CalendarConflict {
        date: r.get("date"),
        status: r.get("status"),
        guest_name: r.get("guest_name"),
        booking_id: r.get("booking_id"),
    }).collect();

    let first_date = &conflicts[0].date;
    let from_naive = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let first_naive = chrono::NaiveDate::parse_from_str(first_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let max_nights = (first_naive - from_naive).num_days() as i32;

    Ok(AvailabilityResult {
        available: false,
        conflicts,
        max_nights: Some(max_nights),
    })
}

#[tauri::command]
pub async fn check_availability(state: State<'_, AppState>, room_id: String, from_date: String, to_date: String) -> Result<AvailabilityResult, String> {
    do_check_availability(&state.db, &room_id, &from_date, &to_date).await
}

// ─── Create Reservation ───

pub async fn do_create_reservation(
    pool: &Pool<Sqlite>,
    app_handle: Option<&tauri::AppHandle>,
    req: CreateReservationRequest,
) -> Result<Booking, String> {
    let booking = reservation_lifecycle::create_reservation(pool, req)
        .await
        .map_err(|error| error.to_string())?;

    if let Some(app) = app_handle {
        emit_db_update(app, "rooms");
    }

    Ok(booking)
}

#[tauri::command]
pub async fn create_reservation(state: State<'_, AppState>, app: tauri::AppHandle, req: CreateReservationRequest) -> Result<Booking, String> {
    do_create_reservation(&state.db, Some(&app), req).await
}

// ─── Confirm Reservation (Check-in from reservation) ───

#[tauri::command]
pub async fn confirm_reservation(state: State<'_, AppState>, app: tauri::AppHandle, booking_id: String) -> Result<Booking, String> {
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    let row = sqlx::query("SELECT room_id, status, deposit_amount, total_price, scheduled_checkin, scheduled_checkout FROM bookings WHERE id = ?")
        .bind(&booking_id)
        .fetch_one(&mut *tx).await.map_err(|e| format!("Booking not found: {}", e))?;

    let status: String = row.get("status");
    if status != "booked" {
        return Err(format!("Booking {} is not in 'booked' status (current: {})", booking_id, status));
    }

    let room_id: String = row.get("room_id");
    let _deposit: f64 = row.try_get::<f64, _>("deposit_amount").unwrap_or(0.0);
    let scheduled_checkin: Option<String> = row.get("scheduled_checkin");
    let scheduled_checkout: Option<String> = row.get("scheduled_checkout");
    let now = chrono::Local::now();
    let _today_str = now.format("%Y-%m-%d").to_string();

    // Recalculate nights and price based on actual check-in (today) vs scheduled checkout
    let checkout_date_str = scheduled_checkout.as_deref().unwrap_or("");
    let checkout_naive = chrono::NaiveDate::parse_from_str(checkout_date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkout date: {}", e))?;
    let today_naive = now.date_naive();

    let actual_nights = (checkout_naive - today_naive).num_days().max(1) as i32;

    // Get room base price
    let price_row = sqlx::query("SELECT base_price FROM rooms WHERE id = ?")
        .bind(&room_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    let base_price = get_f64(&price_row, "base_price");
    let new_total = base_price * actual_nights as f64;

    // Update booking: status, check_in_at, nights, total_price, expected_checkout
    sqlx::query(
        "UPDATE bookings SET status = 'active', check_in_at = ?, nights = ?, total_price = ?, expected_checkout = ?, paid_amount = COALESCE(deposit_amount, 0) WHERE id = ?"
    )
    .bind(now.to_rfc3339())
    .bind(actual_nights)
    .bind(new_total)
    .bind(checkout_date_str)
    .bind(&booking_id)
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Update room status to occupied
    sqlx::query("UPDATE rooms SET status = 'occupied' WHERE id = ?")
        .bind(&room_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Add calendar rows for early check-in days (today → scheduled_checkin)
    let sched_checkin_naive = scheduled_checkin.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    if let Some(sched_date) = sched_checkin_naive {
        if today_naive < sched_date {
            let mut d = today_naive;
            while d < sched_date {
                let ds = d.format("%Y-%m-%d").to_string();
                // Insert only if not already occupied by another booking
                sqlx::query(
                    "INSERT OR IGNORE INTO room_calendar (room_id, date, booking_id, status) VALUES (?, ?, ?, 'occupied')"
                )
                .bind(&room_id).bind(&ds).bind(&booking_id)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                d += chrono::Duration::days(1);
            }
        }
    }

    // Update existing calendar rows from 'booked' to 'occupied'
    sqlx::query("UPDATE room_calendar SET status = 'occupied' WHERE booking_id = ?")
        .bind(&booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Record charge transaction for room revenue (using recalculated total)
    let charge_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
         VALUES (?, ?, ?, 'charge', 'Room charge (reservation)', ?)"
    )
    .bind(&charge_id).bind(&booking_id).bind(new_total).bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    emit_db_update(&app, "rooms");

    super::rooms::get_booking_by_id(&state.db, &booking_id).await
}

// ─── Cancel Reservation ───

pub async fn do_cancel_reservation(
    pool: &Pool<Sqlite>,
    app_handle: Option<&tauri::AppHandle>,
    booking_id: &str,
) -> Result<(), String> {
    reservation_lifecycle::cancel_reservation(pool, booking_id)
        .await
        .map_err(|error| error.to_string())?;

    if let Some(app) = app_handle {
        emit_db_update(app, "rooms");
    }

    Ok(())
}

#[tauri::command]
pub async fn cancel_reservation(state: State<'_, AppState>, app: tauri::AppHandle, booking_id: String) -> Result<(), String> {
    do_cancel_reservation(&state.db, Some(&app), &booking_id).await
}

// ─── Modify Reservation ───

pub async fn do_modify_reservation(pool: &Pool<Sqlite>, app_handle: Option<&tauri::AppHandle>, req: ModifyReservationRequest) -> Result<Booking, String> {
    if req.new_nights <= 0 {
        return Err("Number of nights must be greater than 0".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let row = sqlx::query("SELECT room_id, status FROM bookings WHERE id = ?")
        .bind(&req.booking_id)
        .fetch_one(&mut *tx).await.map_err(|e| format!("Booking not found: {}", e))?;

    let status: String = row.get("status");
    if status != "booked" {
        return Err(format!("Can only modify reservations in 'booked' status (current: {})", status));
    }

    let room_id: String = row.get("room_id");

    sqlx::query("DELETE FROM room_calendar WHERE booking_id = ?")
        .bind(&req.booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let conflicts = sqlx::query(
        "SELECT date FROM room_calendar WHERE room_id = ? AND date >= ? AND date < ?"
    )
    .bind(&room_id).bind(&req.new_check_in_date).bind(&req.new_check_out_date)
    .fetch_all(&mut *tx).await.map_err(|e| e.to_string())?;

    if !conflicts.is_empty() {
        let first: String = conflicts[0].get("date");
        return Err(format!("Room {} is booked on {}. Cannot modify.", room_id, first));
    }

    let price_row = sqlx::query("SELECT base_price FROM rooms WHERE id = ?")
        .bind(&room_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    let base_price = get_f64(&price_row, "base_price");
    let new_total = base_price * req.new_nights as f64;

    sqlx::query(
        "UPDATE bookings SET check_in_at = ?, expected_checkout = ?, scheduled_checkin = ?, scheduled_checkout = ?, nights = ?, total_price = ? WHERE id = ?"
    )
    .bind(&req.new_check_in_date)
    .bind(&req.new_check_out_date)
    .bind(&req.new_check_in_date)
    .bind(&req.new_check_out_date)
    .bind(req.new_nights)
    .bind(new_total)
    .bind(&req.booking_id)
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let from = chrono::NaiveDate::parse_from_str(&req.new_check_in_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let to = chrono::NaiveDate::parse_from_str(&req.new_check_out_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let mut d = from;
    while d < to {
        sqlx::query(
            "INSERT INTO room_calendar (room_id, date, booking_id, status) VALUES (?, ?, ?, 'booked')"
        )
        .bind(&room_id).bind(d.format("%Y-%m-%d").to_string()).bind(&req.booking_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        d += chrono::Duration::days(1);
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    if let Some(app) = app_handle {
        emit_db_update(app, "rooms");
    }

    super::rooms::get_booking_by_id(pool, &req.booking_id).await
}

#[tauri::command]
pub async fn modify_reservation(state: State<'_, AppState>, app: tauri::AppHandle, req: ModifyReservationRequest) -> Result<Booking, String> {
    do_modify_reservation(&state.db, Some(&app), req).await
}

// ─── Get Room Calendar ───

#[tauri::command]
pub async fn get_room_calendar(state: State<'_, AppState>, room_id: String, from: String, to: String) -> Result<Vec<CalendarEntry>, String> {
    let rows = sqlx::query(
        "SELECT room_id, date, booking_id, status FROM room_calendar
         WHERE room_id = ? AND date >= ? AND date <= ?
         ORDER BY date ASC"
    )
    .bind(&room_id).bind(&from).bind(&to)
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|r| CalendarEntry {
        room_id: r.get("room_id"),
        date: r.get("date"),
        booking_id: r.get("booking_id"),
        status: r.get("status"),
    }).collect())
}

// ─── Get Rooms Availability (Dashboard) ───

pub async fn do_get_rooms_availability(pool: &Pool<Sqlite>) -> Result<Vec<RoomWithAvailability>, String> {
    let room_rows = sqlx::query("SELECT id, name, type, floor, has_balcony, base_price, max_guests, extra_person_fee, status FROM rooms ORDER BY id")
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut results = Vec::new();

    for rr in &room_rows {
        let room = Room {
            id: rr.get("id"),
            name: rr.get("name"),
            room_type: rr.get("type"),
            floor: rr.get("floor"),
            has_balcony: rr.get::<i32, _>("has_balcony") == 1,
            base_price: get_f64(rr, "base_price"),
            max_guests: rr.try_get::<i32, _>("max_guests").unwrap_or(2),
            extra_person_fee: rr.try_get::<f64, _>("extra_person_fee").unwrap_or(0.0),
            status: rr.get("status"),
        };

        let current_booking = sqlx::query(
            "SELECT * FROM bookings WHERE room_id = ? AND status = 'active' LIMIT 1"
        )
        .bind(&room.id)
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

        let res_rows = sqlx::query(
            "SELECT b.id, g.full_name, b.scheduled_checkin, b.scheduled_checkout, b.deposit_amount, b.status
             FROM bookings b
             JOIN guests g ON g.id = b.primary_guest_id
             WHERE b.room_id = ? AND b.status = 'booked' AND b.scheduled_checkin >= ?
             ORDER BY b.scheduled_checkin ASC"
        )
        .bind(&room.id).bind(&today)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        let upcoming: Vec<UpcomingReservation> = res_rows.iter().map(|r| UpcomingReservation {
            booking_id: r.get("id"),
            guest_name: r.get("full_name"),
            scheduled_checkin: r.get::<Option<String>, _>("scheduled_checkin").unwrap_or_default(),
            scheduled_checkout: r.get::<Option<String>, _>("scheduled_checkout").unwrap_or_default(),
            deposit_amount: r.try_get::<f64, _>("deposit_amount").unwrap_or(0.0),
            status: r.get("status"),
        }).collect();

        let next_until = upcoming.first().map(|u| u.scheduled_checkin.clone());

        results.push(RoomWithAvailability {
            room,
            current_booking,
            upcoming_reservations: upcoming,
            next_available_until: next_until,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_rooms_availability(state: State<'_, AppState>) -> Result<Vec<RoomWithAvailability>, String> {
    do_get_rooms_availability(&state.db).await
}
