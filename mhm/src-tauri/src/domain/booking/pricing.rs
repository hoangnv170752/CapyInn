use sqlx::{Pool, Sqlite};

use crate::commands::pricing::do_calculate_price_preview;

use super::{BookingError, BookingResult};

pub async fn calculate_stay_price(
    pool: &Pool<Sqlite>,
    room_id: &str,
    check_in: &str,
    check_out: &str,
    pricing_type: &str,
) -> BookingResult<crate::pricing::PricingResult> {
    let room_type = sqlx::query_scalar::<_, String>("SELECT type FROM rooms WHERE id = ? LIMIT 1")
        .bind(room_id)
        .fetch_optional(pool)
        .await
        .map_err(BookingError::from)?
        .ok_or_else(|| BookingError::not_found(format!("Không tìm thấy phòng {}", room_id)))?;

    do_calculate_price_preview(pool, &room_type, check_in, check_out, pricing_type)
        .await
        .map_err(BookingError::pricing)
}
