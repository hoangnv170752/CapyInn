use chrono::{Local, NaiveDate};
use sqlx::{Pool, Row, Sqlite};

use crate::{
    domain::booking::{pricing::calculate_stay_price_tx, BookingError, BookingResult},
    models::{status, Booking, CreateReservationRequest},
};

use super::{
    billing_service::{record_cancellation_fee_tx, record_deposit_tx},
    support::begin_tx,
};

pub async fn create_reservation(
    pool: &Pool<Sqlite>,
    req: CreateReservationRequest,
) -> BookingResult<Booking> {
    if req.nights <= 0 {
        return Err(BookingError::validation(
            "Number of nights must be greater than 0".to_string(),
        ));
    }

    let mut tx = begin_tx(pool).await?;

    let conflicts = sqlx::query(
        "SELECT date FROM room_calendar WHERE room_id = ? AND date >= ? AND date < ? ORDER BY date ASC",
    )
    .bind(&req.room_id)
    .bind(&req.check_in_date)
    .bind(&req.check_out_date)
    .fetch_all(&mut *tx)
    .await?;

    if let Some(first_conflict) = conflicts.first() {
        let first_date: String = first_conflict.get("date");
        return Err(BookingError::conflict(format!(
            "Room {} is booked on {}. Cannot create reservation.",
            req.room_id, first_date
        )));
    }

    let now = Local::now().to_rfc3339();
    let deposit_amount = req.deposit_amount.unwrap_or(0.0);
    let pricing = calculate_stay_price_tx(
        &mut tx,
        &req.room_id,
        &req.check_in_date,
        &req.check_out_date,
        "nightly",
    )
    .await?;

    let guest_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO guests (id, guest_type, full_name, doc_number, phone, created_at)
         VALUES (?, 'domestic', ?, ?, ?, ?)",
    )
    .bind(&guest_id)
    .bind(&req.guest_name)
    .bind(req.guest_doc_number.as_deref().unwrap_or(""))
    .bind(req.guest_phone.as_deref())
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    let booking_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bookings (
            id, room_id, primary_guest_id, check_in_at, expected_checkout, actual_checkout,
            nights, total_price, paid_amount, status, source, notes, created_by,
            booking_type, pricing_type, deposit_amount, guest_phone, scheduled_checkin,
            scheduled_checkout, pricing_snapshot, created_at
         ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?, 0, ?, ?, ?, NULL, 'reservation', 'nightly', ?, ?, ?, ?, NULL, ?)",
    )
    .bind(&booking_id)
    .bind(&req.room_id)
    .bind(&guest_id)
    .bind(&req.check_in_date)
    .bind(&req.check_out_date)
    .bind(req.nights)
    .bind(pricing.total)
    .bind(status::booking::BOOKED)
    .bind(req.source.as_deref().unwrap_or("phone"))
    .bind(req.notes.as_deref())
    .bind(deposit_amount)
    .bind(req.guest_phone.as_deref())
    .bind(&req.check_in_date)
    .bind(&req.check_out_date)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO booking_guests (booking_id, guest_id) VALUES (?, ?)")
        .bind(&booking_id)
        .bind(&guest_id)
        .execute(&mut *tx)
        .await?;

    insert_booked_calendar_rows(
        &mut tx,
        &req.room_id,
        &booking_id,
        &req.check_in_date,
        &req.check_out_date,
    )
    .await?;

    if deposit_amount > 0.0 {
        record_deposit_tx(&mut tx, &booking_id, deposit_amount, "Reservation deposit").await?;
    }

    tx.commit().await.map_err(BookingError::from)?;

    fetch_booking(pool, &booking_id).await
}

pub async fn cancel_reservation(pool: &Pool<Sqlite>, booking_id: &str) -> BookingResult<()> {
    let mut tx = begin_tx(pool).await?;

    let booking = sqlx::query(
        "SELECT room_id, status, COALESCE(deposit_amount, 0) AS deposit_amount
         FROM bookings
         WHERE id = ?",
    )
    .bind(booking_id)
    .fetch_optional(&mut *tx)
    .await?;

    let booking = booking
        .ok_or_else(|| BookingError::not_found(format!("Booking not found: {}", booking_id)))?;

    let status: String = booking.get("status");
    if status != status::booking::BOOKED {
        return Err(BookingError::conflict(format!(
            "Can only cancel reservations in 'booked' status (current: {})",
            status
        )));
    }

    let room_id: String = booking.get("room_id");
    let deposit_amount: f64 = booking.get("deposit_amount");

    sqlx::query("UPDATE bookings SET status = ? WHERE id = ?")
        .bind(status::booking::CANCELLED)
        .bind(booking_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM room_calendar WHERE booking_id = ? AND status = ?")
        .bind(booking_id)
        .bind(status::calendar::BOOKED)
        .execute(&mut *tx)
        .await?;

    if deposit_amount > 0.0 {
        record_cancellation_fee_tx(
            &mut tx,
            booking_id,
            deposit_amount,
            "Deposit retained (cancellation)",
        )
        .await?;
    }

    let remaining_booked: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM room_calendar WHERE room_id = ? AND status = ?")
            .bind(&room_id)
            .bind(status::calendar::BOOKED)
            .fetch_one(&mut *tx)
            .await?;

    let room_status = sqlx::query_scalar::<_, String>("SELECT status FROM rooms WHERE id = ?")
        .bind(&room_id)
        .fetch_one(&mut *tx)
        .await?;

    if room_status == status::room::BOOKED && remaining_booked.0 == 0 {
        sqlx::query("UPDATE rooms SET status = ? WHERE id = ?")
            .bind(status::room::VACANT)
            .bind(&room_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await.map_err(BookingError::from)?;

    Ok(())
}

async fn insert_booked_calendar_rows(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    room_id: &str,
    booking_id: &str,
    check_in_date: &str,
    check_out_date: &str,
) -> BookingResult<()> {
    let from = parse_date(check_in_date)?;
    let to = parse_date(check_out_date)?;
    let mut date = from;

    while date < to {
        sqlx::query(
            "INSERT INTO room_calendar (room_id, date, booking_id, status) VALUES (?, ?, ?, ?)",
        )
        .bind(room_id)
        .bind(date.format("%Y-%m-%d").to_string())
        .bind(booking_id)
        .bind(status::calendar::BOOKED)
        .execute(&mut **tx)
        .await?;

        date += chrono::Duration::days(1);
    }

    Ok(())
}

async fn fetch_booking(pool: &Pool<Sqlite>, booking_id: &str) -> BookingResult<Booking> {
    let row = sqlx::query(
        "SELECT id, room_id, primary_guest_id, check_in_at, expected_checkout, actual_checkout,
                nights, total_price, paid_amount, status, source, notes, created_at
         FROM bookings
         WHERE id = ?",
    )
    .bind(booking_id)
    .fetch_optional(pool)
    .await?;

    let row =
        row.ok_or_else(|| BookingError::not_found(format!("Booking not found: {}", booking_id)))?;

    Ok(Booking {
        id: row.get("id"),
        room_id: row.get("room_id"),
        primary_guest_id: row.get("primary_guest_id"),
        check_in_at: row.get("check_in_at"),
        expected_checkout: row.get("expected_checkout"),
        actual_checkout: row.get("actual_checkout"),
        nights: row.get("nights"),
        total_price: read_f64(&row, "total_price"),
        paid_amount: read_f64(&row, "paid_amount"),
        status: row.get("status"),
        source: row.get("source"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
    })
}

fn parse_date(value: &str) -> BookingResult<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|error| BookingError::datetime_parse(error.to_string()))
}

fn read_f64(row: &sqlx::sqlite::SqliteRow, column: &str) -> f64 {
    row.try_get::<f64, _>(column)
        .unwrap_or_else(|_| row.get::<i64, _>(column) as f64)
}
