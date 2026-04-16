use sqlx::{Pool, Sqlite};

use crate::{domain::booking::BookingResult, models::FolioLine};

pub async fn insert_folio_line(
    pool: &Pool<Sqlite>,
    booking_id: &str,
    category: &str,
    description: &str,
    amount: f64,
    created_by: Option<&str>,
    created_at: &str,
) -> BookingResult<FolioLine> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO folio_lines (id, booking_id, category, description, amount, created_by, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(booking_id)
    .bind(category)
    .bind(description)
    .bind(amount)
    .bind(created_by)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(FolioLine {
        id,
        booking_id: booking_id.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        amount,
        created_by: created_by.map(str::to_string),
        created_at: created_at.to_string(),
    })
}
