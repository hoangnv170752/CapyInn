use sqlx::{Pool, Sqlite, Row};
use tauri::State;
use crate::models::*;
use super::{AppState, get_f64, emit_db_update, get_user_id};

// ─── Group Booking Commands ───

/// Check-in a group: creates booking_groups + N bookings atomically
#[tauri::command]
pub async fn group_checkin(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    req: GroupCheckinRequest,
) -> Result<BookingGroup, String> {
    if req.room_ids.is_empty() {
        return Err("Phải chọn ít nhất 1 phòng".to_string());
    }
    if req.nights <= 0 {
        return Err("Số đêm phải > 0".to_string());
    }
    if !req.room_ids.contains(&req.master_room_id) {
        return Err("Phòng đại diện phải nằm trong danh sách phòng".to_string());
    }

    let user_id = get_user_id(&state);
    let now = chrono::Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();
    let is_reservation = if let Some(ref d) = req.check_in_date {
        d != &today_str
    } else {
        false
    };
    let checkin_date = req.check_in_date.clone().unwrap_or(today_str);
    let checkin_naive = chrono::NaiveDate::parse_from_str(&checkin_date, "%Y-%m-%d")
        .map_err(|_| "Ngày check-in không hợp lệ".to_string())?;
    let checkout_naive = checkin_naive + chrono::Duration::days(req.nights as i64);
    let checkout_date = checkout_naive.format("%Y-%m-%d").to_string();

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // Validate rooms: check-in today requires vacant, reservation checks calendar overlap
    for room_id in &req.room_ids {
        let status: (String,) = sqlx::query_as("SELECT status FROM rooms WHERE id = ?")
            .bind(room_id)
            .fetch_one(&mut *tx).await
            .map_err(|_| format!("Phòng {} không tồn tại", room_id))?;
        if !is_reservation && status.0 != "vacant" {
            return Err(format!("Phòng {} không trống (status: {})", room_id, status.0));
        }

        // Calendar overlap check for the target date range
        let conflicts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM room_calendar WHERE room_id = ? AND date >= ? AND date < ?"
        )
        .bind(room_id).bind(&checkin_date).bind(&checkout_date)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

        if conflicts.0 > 0 {
            return Err(format!("Phòng {} có lịch trùng trong khoảng ngày đã chọn", room_id));
        }
    }

    // Create booking_groups record
    let group_id = uuid::Uuid::new_v4().to_string();
    let group_status = if is_reservation { "booked" } else { "active" };
    sqlx::query(
        "INSERT INTO booking_groups (id, group_name, organizer_name, organizer_phone, total_rooms, status, notes, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&group_id)
    .bind(&req.group_name)
    .bind(&req.organizer_name)
    .bind(&req.organizer_phone)
    .bind(req.room_ids.len() as i32)
    .bind(group_status)
    .bind(&req.notes)
    .bind(&user_id)
    .bind(now.to_rfc3339())
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let paid = req.paid_amount.unwrap_or(0.0);
    let paid_per_room = if !req.room_ids.is_empty() { paid / req.room_ids.len() as f64 } else { 0.0 };
    let mut master_booking_id: Option<String> = None;

    // Create bookings for each room
    for room_id in &req.room_ids {
        let is_master = room_id == &req.master_room_id;
        let room_guests = req.guests_per_room.get(room_id.as_str()).cloned().unwrap_or_default();

        // Get room price
        let price_row = sqlx::query("SELECT base_price FROM rooms WHERE id = ?")
            .bind(room_id)
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
        let room_price = get_f64(&price_row, "base_price");
        let total_price = room_price * req.nights as f64;

        // Create primary guest (or placeholder)
        let primary_guest_id = uuid::Uuid::new_v4().to_string();
        if let Some(primary) = room_guests.first() {
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
        } else {
            // Placeholder guest for rooms without guest info
            sqlx::query(
                "INSERT INTO guests (id, guest_type, full_name, doc_number, created_at)
                 VALUES (?, 'domestic', ?, '', ?)"
            )
            .bind(&primary_guest_id)
            .bind(format!("Khách đoàn {} - {}", req.group_name, room_id))
            .bind(now.to_rfc3339())
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }

        // Create booking
        let booking_id = uuid::Uuid::new_v4().to_string();
        let booking_status = if is_reservation { "booked" } else { "active" };
        let booking_type = if is_reservation { "reservation" } else { "walk-in" };
        let checkin_at = if is_reservation {
            format!("{}T14:00:00+07:00", &checkin_date)
        } else {
            now.to_rfc3339()
        };
        let checkout_at = if is_reservation {
            format!("{}T12:00:00+07:00", &checkout_date)
        } else {
            let co = now + chrono::Duration::days(req.nights as i64);
            co.to_rfc3339()
        };
        let sched_checkin: Option<&str> = if is_reservation { Some(&checkin_date) } else { None };
        let sched_checkout: Option<&str> = if is_reservation { Some(&checkout_date) } else { None };
        sqlx::query(
            "INSERT INTO bookings (id, room_id, primary_guest_id, check_in_at, expected_checkout, nights, total_price, paid_amount, status, source, notes, created_by, booking_type, group_id, is_master_room, scheduled_checkin, scheduled_checkout, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&booking_id)
        .bind(room_id)
        .bind(&primary_guest_id)
        .bind(&checkin_at)
        .bind(&checkout_at)
        .bind(req.nights)
        .bind(total_price)
        .bind(paid_per_room)
        .bind(booking_status)
        .bind(req.source.as_deref().unwrap_or("walk-in"))
        .bind(&req.notes)
        .bind(&user_id)
        .bind(booking_type)
        .bind(&group_id)
        .bind(if is_master { 1 } else { 0 })
        .bind(sched_checkin)
        .bind(sched_checkout)
        .bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        if is_master {
            master_booking_id = Some(booking_id.clone());
        }

        // Link primary guest
        sqlx::query("INSERT INTO booking_guests (booking_id, guest_id) VALUES (?, ?)")
            .bind(&booking_id).bind(&primary_guest_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // Additional guests
        for guest_req in room_guests.iter().skip(1) {
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

        // Record charge + payment only for immediate check-in
        if !is_reservation {
            let charge_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
                 VALUES (?, ?, ?, 'charge', 'Tiền phòng (đoàn)', ?)"
            )
            .bind(&charge_id).bind(&booking_id).bind(total_price).bind(now.to_rfc3339())
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

            if paid_per_room > 0.0 {
                let txn_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
                     VALUES (?, ?, ?, 'payment', 'Thanh toán group check-in', ?)"
                )
                .bind(&txn_id).bind(&booking_id).bind(paid_per_room).bind(now.to_rfc3339())
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
            }
        } else if paid_per_room > 0.0 {
            // Reservation deposit
            let txn_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
                 VALUES (?, ?, ?, 'payment', 'Đặt cọc đoàn', ?)"
            )
            .bind(&txn_id).bind(&booking_id).bind(paid_per_room).bind(now.to_rfc3339())
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }

        // Calendar blocking
        let calendar_status = if is_reservation { "booked" } else { "occupied" };
        let mut d = checkin_naive;
        while d < checkout_naive {
            sqlx::query(
                "INSERT OR REPLACE INTO room_calendar (room_id, date, booking_id, status) VALUES (?, ?, ?, ?)"
            )
            .bind(room_id).bind(d.format("%Y-%m-%d").to_string()).bind(&booking_id).bind(calendar_status)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
            d += chrono::Duration::days(1);
        }

        // Update room status only for immediate check-in
        if !is_reservation {
            sqlx::query("UPDATE rooms SET status = 'occupied' WHERE id = ?")
                .bind(room_id)
                .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
    }

    // Set master_booking_id on group
    if let Some(ref mid) = master_booking_id {
        sqlx::query("UPDATE booking_groups SET master_booking_id = ? WHERE id = ?")
            .bind(mid).bind(&group_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    emit_db_update(&app, "rooms");

    Ok(BookingGroup {
        id: group_id,
        group_name: req.group_name,
        master_booking_id,
        organizer_name: req.organizer_name,
        organizer_phone: req.organizer_phone,
        total_rooms: req.room_ids.len() as i32,
        status: group_status.to_string(),
        notes: req.notes,
        created_by: user_id,
        created_at: now.to_rfc3339(),
    })
}

/// Checkout subset of group rooms
#[tauri::command]
pub async fn group_checkout(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    req: GroupCheckoutRequest,
) -> Result<(), String> {
    if req.booking_ids.is_empty() {
        return Err("Phải chọn ít nhất 1 phòng để checkout".to_string());
    }

    let now = chrono::Local::now();
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    for booking_id in &req.booking_ids {
        let row = sqlx::query("SELECT room_id, paid_amount FROM bookings WHERE id = ? AND status = 'active' AND group_id = ?")
            .bind(booking_id).bind(&req.group_id)
            .fetch_one(&mut *tx).await
            .map_err(|_| format!("Booking {} không tìm thấy hoặc đã checkout", booking_id))?;

        let room_id: String = row.get("room_id");

        // Update booking status
        sqlx::query("UPDATE bookings SET status = 'checked_out', actual_checkout = ? WHERE id = ?")
            .bind(now.to_rfc3339()).bind(booking_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // Room → cleaning
        sqlx::query("UPDATE rooms SET status = 'cleaning' WHERE id = ?")
            .bind(&room_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // Housekeeping task
        let hk_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO housekeeping (id, room_id, status, triggered_at, created_at)
             VALUES (?, ?, 'needs_cleaning', ?, ?)"
        )
        .bind(&hk_id).bind(&room_id).bind(now.to_rfc3339()).bind(now.to_rfc3339())
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // Clear calendar
        sqlx::query("DELETE FROM room_calendar WHERE booking_id = ?")
            .bind(booking_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // Check if master was checked out → auto-transfer
    let master_row = sqlx::query("SELECT master_booking_id FROM booking_groups WHERE id = ?")
        .bind(&req.group_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    let current_master: Option<String> = master_row.get("master_booking_id");

    if let Some(ref mid) = current_master {
        if req.booking_ids.contains(mid) {
            // Master was checked out — find first remaining active booking
            let next_master = sqlx::query_as::<_, (String,)>(
                "SELECT id FROM bookings WHERE group_id = ? AND status = 'active' LIMIT 1"
            )
            .bind(&req.group_id)
            .fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?;

            if let Some((new_mid,)) = next_master {
                sqlx::query("UPDATE bookings SET is_master_room = 0 WHERE group_id = ?")
                    .bind(&req.group_id)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                sqlx::query("UPDATE bookings SET is_master_room = 1 WHERE id = ?")
                    .bind(&new_mid)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                sqlx::query("UPDATE booking_groups SET master_booking_id = ? WHERE id = ?")
                    .bind(&new_mid).bind(&req.group_id)
                    .execute(&mut *tx).await.map_err(|e| e.to_string())?;
            }
        }
    }

    // Update group status
    let remaining: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bookings WHERE group_id = ? AND status = 'active'"
    )
    .bind(&req.group_id)
    .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

    let new_status = if remaining.0 == 0 { "completed" } else { "partial_checkout" };
    sqlx::query("UPDATE booking_groups SET status = ? WHERE id = ?")
        .bind(new_status).bind(&req.group_id)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Record final payment if provided
    if let Some(final_paid) = req.final_paid {
        if final_paid > 0.0 {
            // Apply to first active booking or last checked-out
            let target_booking: (String,) = sqlx::query_as(
                "SELECT id FROM bookings WHERE group_id = ? ORDER BY CASE WHEN status = 'active' THEN 0 ELSE 1 END, created_at ASC LIMIT 1"
            )
            .bind(&req.group_id)
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

            let txn_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
                 VALUES (?, ?, ?, 'payment', 'Thanh toán group checkout', ?)"
            )
            .bind(&txn_id).bind(&target_booking.0).bind(final_paid).bind(now.to_rfc3339())
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    emit_db_update(&app, "rooms");

    Ok(())
}

/// Get group detail with bookings, services, totals
pub async fn do_get_group_detail(pool: &Pool<Sqlite>, group_id: &str) -> Result<GroupDetailResponse, String> {
    let row = sqlx::query("SELECT * FROM booking_groups WHERE id = ?")
        .bind(group_id)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    let group = BookingGroup {
        id: row.get("id"),
        group_name: row.get("group_name"),
        master_booking_id: row.get("master_booking_id"),
        organizer_name: row.get("organizer_name"),
        organizer_phone: row.get("organizer_phone"),
        total_rooms: row.get("total_rooms"),
        status: row.get("status"),
        notes: row.get("notes"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    };

    // Get bookings
    let booking_rows = sqlx::query(
        "SELECT b.id, b.room_id, r.name as room_name, g.full_name as guest_name,
                b.check_in_at, b.expected_checkout, b.actual_checkout, b.nights,
                b.total_price, b.paid_amount, b.status, b.source,
                b.booking_type, b.deposit_amount, b.scheduled_checkin, b.scheduled_checkout, b.guest_phone
         FROM bookings b
         JOIN rooms r ON r.id = b.room_id
         JOIN guests g ON g.id = b.primary_guest_id
         WHERE b.group_id = ?
         ORDER BY r.floor, r.id"
    )
    .bind(group_id)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    let bookings: Vec<BookingWithGuest> = booking_rows.iter().map(|r| BookingWithGuest {
        id: r.get("id"),
        room_id: r.get("room_id"),
        room_name: r.get("room_name"),
        guest_name: r.get("guest_name"),
        check_in_at: r.get("check_in_at"),
        expected_checkout: r.get("expected_checkout"),
        actual_checkout: r.get("actual_checkout"),
        nights: r.get("nights"),
        total_price: get_f64(r, "total_price"),
        paid_amount: get_f64(r, "paid_amount"),
        status: r.get("status"),
        source: r.get("source"),
        booking_type: r.get("booking_type"),
        deposit_amount: r.try_get::<f64, _>("deposit_amount").ok(),
        scheduled_checkin: r.get("scheduled_checkin"),
        scheduled_checkout: r.get("scheduled_checkout"),
        guest_phone: r.get("guest_phone"),
    }).collect();

    // Get services
    let service_rows = sqlx::query("SELECT * FROM group_services WHERE group_id = ? ORDER BY created_at")
        .bind(group_id)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    let services: Vec<GroupService> = service_rows.iter().map(|r| GroupService {
        id: r.get("id"),
        group_id: r.get("group_id"),
        booking_id: r.get("booking_id"),
        name: r.get("name"),
        quantity: r.get("quantity"),
        unit_price: get_f64(r, "unit_price"),
        total_price: get_f64(r, "total_price"),
        note: r.get("note"),
        created_by: r.get("created_by"),
        created_at: r.get("created_at"),
    }).collect();

    let total_room_cost: f64 = bookings.iter().map(|b| b.total_price).sum();
    let total_service_cost: f64 = services.iter().map(|s| s.total_price).sum();
    let paid_amount: f64 = bookings.iter().map(|b| b.paid_amount).sum();

    Ok(GroupDetailResponse {
        group,
        bookings,
        services,
        total_room_cost,
        total_service_cost,
        grand_total: total_room_cost + total_service_cost,
        paid_amount,
    })
}

#[tauri::command]
pub async fn get_group_detail(state: State<'_, AppState>, group_id: String) -> Result<GroupDetailResponse, String> {
    do_get_group_detail(&state.db, &group_id).await
}

/// List all groups, optionally filtered by status
#[tauri::command]
pub async fn get_all_groups(state: State<'_, AppState>, status: Option<String>) -> Result<Vec<BookingGroup>, String> {
    let rows = if let Some(ref s) = status {
        sqlx::query("SELECT * FROM booking_groups WHERE status = ? ORDER BY created_at DESC")
            .bind(s)
            .fetch_all(&state.db).await.map_err(|e| e.to_string())?
    } else {
        sqlx::query("SELECT * FROM booking_groups ORDER BY created_at DESC")
            .fetch_all(&state.db).await.map_err(|e| e.to_string())?
    };

    Ok(rows.iter().map(|r| BookingGroup {
        id: r.get("id"),
        group_name: r.get("group_name"),
        master_booking_id: r.get("master_booking_id"),
        organizer_name: r.get("organizer_name"),
        organizer_phone: r.get("organizer_phone"),
        total_rooms: r.get("total_rooms"),
        status: r.get("status"),
        notes: r.get("notes"),
        created_by: r.get("created_by"),
        created_at: r.get("created_at"),
    }).collect())
}

/// Add a group service (laundry, tour, motorbike, etc.)
#[tauri::command]
pub async fn add_group_service(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    req: AddGroupServiceRequest,
) -> Result<GroupService, String> {
    let user_id = get_user_id(&state);
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();
    let total_price = req.quantity as f64 * req.unit_price;

    sqlx::query(
        "INSERT INTO group_services (id, group_id, booking_id, name, quantity, unit_price, total_price, note, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.group_id)
    .bind(&req.booking_id)
    .bind(&req.name)
    .bind(req.quantity)
    .bind(req.unit_price)
    .bind(total_price)
    .bind(&req.note)
    .bind(&user_id)
    .bind(&now)
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    emit_db_update(&app, "groups");

    Ok(GroupService {
        id,
        group_id: req.group_id,
        booking_id: req.booking_id,
        name: req.name,
        quantity: req.quantity,
        unit_price: req.unit_price,
        total_price,
        note: req.note,
        created_by: user_id,
        created_at: now,
    })
}

/// Remove a group service
#[tauri::command]
pub async fn remove_group_service(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    service_id: String,
) -> Result<(), String> {
    sqlx::query("DELETE FROM group_services WHERE id = ?")
        .bind(&service_id)
        .execute(&state.db).await.map_err(|e| e.to_string())?;

    emit_db_update(&app, "groups");
    Ok(())
}

/// Auto-assign rooms: prefer same floor, greedy fill
#[tauri::command]
pub async fn auto_assign_rooms(
    state: State<'_, AppState>,
    req: AutoAssignRequest,
) -> Result<AutoAssignResult, String> {
    if req.room_count <= 0 {
        return Err("Số phòng phải > 0".to_string());
    }

    let rows = if let Some(ref rt) = req.room_type {
        sqlx::query("SELECT * FROM rooms WHERE status = 'vacant' AND type = ? ORDER BY floor, id")
            .bind(rt)
            .fetch_all(&state.db).await.map_err(|e| e.to_string())?
    } else {
        sqlx::query("SELECT * FROM rooms WHERE status = 'vacant' ORDER BY floor, id")
            .fetch_all(&state.db).await.map_err(|e| e.to_string())?
    };

    let vacant_rooms: Vec<Room> = rows.iter().map(|r| Room {
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

    if vacant_rooms.len() < req.room_count as usize {
        return Err(format!(
            "Chỉ có {} phòng trống, cần {} phòng",
            vacant_rooms.len(), req.room_count
        ));
    }

    // Group by floor, sort by count descending (greedy fill)
    let mut floor_groups: std::collections::HashMap<i32, Vec<&Room>> = std::collections::HashMap::new();
    for room in &vacant_rooms {
        floor_groups.entry(room.floor).or_default().push(room);
    }

    let mut floors_sorted: Vec<(i32, Vec<&Room>)> = floor_groups.into_iter().collect();
    floors_sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let mut assignments = Vec::new();
    let needed = req.room_count as usize;

    for (floor, rooms) in &floors_sorted {
        if assignments.len() >= needed {
            break;
        }
        for room in rooms {
            if assignments.len() >= needed {
                break;
            }
            assignments.push(RoomAssignment {
                room: (*room).clone(),
                floor: *floor,
            });
        }
    }

    Ok(AutoAssignResult { assignments })
}

/// Generate group invoice data
pub async fn do_generate_group_invoice(pool: &Pool<Sqlite>, group_id: &str) -> Result<GroupInvoiceData, String> {
    let detail = do_get_group_detail(pool, group_id).await?;

    // Get hotel info from settings
    let hotel_info = sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'hotel_info'")
        .fetch_optional(pool).await.map_err(|e| e.to_string())?;

    let (hotel_name, hotel_address, hotel_phone) = if let Some((val,)) = hotel_info {
        let parsed: serde_json::Value = serde_json::from_str(&val).unwrap_or_default();
        (
            parsed["name"].as_str().unwrap_or("MHM Hotel").to_string(),
            parsed["address"].as_str().unwrap_or("").to_string(),
            parsed["phone"].as_str().unwrap_or("").to_string(),
        )
    } else {
        ("MHM Hotel".to_string(), String::new(), String::new())
    };

    // Build room lines
    let rooms: Vec<GroupInvoiceRoomLine> = detail.bookings.iter().map(|b| {
        let price_per_night = if b.nights > 0 { b.total_price / b.nights as f64 } else { b.total_price };
        GroupInvoiceRoomLine {
            room_name: b.room_name.clone(),
            room_type: String::new(), // simplified
            nights: b.nights,
            price_per_night,
            total: b.total_price,
            guest_name: b.guest_name.clone(),
        }
    }).collect();

    Ok(GroupInvoiceData {
        group: detail.group,
        rooms,
        services: detail.services,
        subtotal_rooms: detail.total_room_cost,
        subtotal_services: detail.total_service_cost,
        grand_total: detail.grand_total,
        paid_amount: detail.paid_amount,
        balance_due: detail.grand_total - detail.paid_amount,
        hotel_name,
        hotel_address,
        hotel_phone,
    })
}

#[tauri::command]
pub async fn generate_group_invoice(state: State<'_, AppState>, group_id: String) -> Result<GroupInvoiceData, String> {
    do_generate_group_invoice(&state.db, &group_id).await
}
