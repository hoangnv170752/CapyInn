use chrono::{DateTime, FixedOffset, Local};
use sqlx::{Pool, Sqlite, Transaction};

use crate::domain::booking::{BookingError, BookingResult};

pub async fn begin_tx<'a>(pool: &'a Pool<Sqlite>) -> BookingResult<Transaction<'a, Sqlite>> {
    pool.begin().await.map_err(BookingError::from)
}

pub fn rfc3339_now() -> String {
    Local::now().to_rfc3339()
}

pub fn parse_booking_datetime(value: &str) -> BookingResult<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|error| BookingError::datetime_parse(error.to_string()))
}
