use sqlx::Row;
use tauri::State;
use crate::models::*;
use super::{AppState, get_f64};

// ─── A3: Get Analytics ───

#[tauri::command]
pub async fn get_analytics(state: State<'_, AppState>, period: String) -> Result<AnalyticsData, String> {
    let now = chrono::Local::now();
    let days = match period.as_str() {
        "30d" => 30,
        "90d" => 90,
        _ => 7,
    };
    let from = (now - chrono::Duration::days(days)).format("%Y-%m-%d").to_string();
    let to = now.format("%Y-%m-%d").to_string();

    // Total revenue (from charge transactions = room revenue)
    let rev: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(amount), 0) AS REAL) FROM transactions WHERE type = 'charge' AND DATE(created_at) >= ? AND DATE(created_at) <= ?"
    ).bind(&from).bind(&to).fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    // Rooms sold (distinct room bookings in period)
    let rooms_sold: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT room_id) FROM bookings WHERE DATE(check_in_at) >= ?"
    ).bind(&from).fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let total_rooms: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rooms")
        .fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let total_nights: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(nights), 0) AS REAL) FROM bookings WHERE DATE(check_in_at) >= ?"
    ).bind(&from).fetch_one(&state.db).await.map_err(|e| e.to_string())?;

    let occupancy_rate = if total_rooms.0 > 0 { (rooms_sold.0 as f64 / total_rooms.0 as f64) * 100.0 } else { 0.0 };
    let adr = if total_nights.0 > 0.0 { rev.0 / total_nights.0 } else { 0.0 };
    let revpar = if total_rooms.0 > 0 { rev.0 / (total_rooms.0 as f64 * days as f64) } else { 0.0 };

    // Daily revenue
    let daily_rows = sqlx::query(
        "SELECT DATE(created_at) as d, SUM(amount) as rev FROM transactions
         WHERE type = 'charge' AND DATE(created_at) >= ? AND DATE(created_at) <= ?
         GROUP BY DATE(created_at) ORDER BY d"
    ).bind(&from).bind(&to).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let daily_revenue = daily_rows.iter().map(|r| DailyRevenue {
        date: r.get("d"),
        revenue: get_f64(r, "rev"),
    }).collect();

    // Revenue by source
    let source_rows = sqlx::query(
        "SELECT COALESCE(b.source, 'walk-in') as src, COALESCE(SUM(b.total_price), 0) as val
         FROM bookings b WHERE DATE(b.check_in_at) >= ?
         GROUP BY src ORDER BY val DESC"
    ).bind(&from).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let revenue_by_source = source_rows.iter().map(|r| SourceRevenue {
        name: r.get("src"),
        value: get_f64(r, "val"),
    }).collect();

    // Expenses by category
    let expense_rows = sqlx::query(
        "SELECT category, COALESCE(SUM(amount), 0) as amt FROM expenses
         WHERE DATE(expense_date) >= ? GROUP BY category ORDER BY amt DESC"
    ).bind(&from).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let expenses_by_category = expense_rows.iter().map(|r| CategoryExpense {
        category: r.get("category"),
        amount: get_f64(r, "amt"),
    }).collect();

    // Top rooms
    let room_rows = sqlx::query(
        "SELECT b.room_id as room, COALESCE(SUM(b.total_price), 0) as rev
         FROM bookings b WHERE DATE(b.check_in_at) >= ?
         GROUP BY b.room_id ORDER BY rev DESC LIMIT 5"
    ).bind(&from).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let top_rooms = room_rows.iter().map(|r| RoomRevenue {
        room: r.get("room"),
        revenue: get_f64(r, "rev"),
    }).collect();

    Ok(AnalyticsData {
        total_revenue: rev.0,
        occupancy_rate,
        adr,
        revpar,
        daily_revenue,
        revenue_by_source,
        expenses_by_category,
        top_rooms,
    })
}

// ─── A4: Get Recent Activity (Dashboard) ───

#[tauri::command]
pub async fn get_recent_activity(state: State<'_, AppState>, limit: i32) -> Result<Vec<ActivityItem>, String> {
    let mut activities: Vec<ActivityItem> = vec![];

    // Recent check-ins
    let checkins = sqlx::query(
        "SELECT b.room_id, g.full_name, b.check_in_at
         FROM bookings b JOIN guests g ON g.id = b.primary_guest_id
         ORDER BY b.check_in_at DESC LIMIT ?"
    ).bind(limit).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for r in &checkins {
        let room_id: String = r.get("room_id");
        let name: String = r.get("full_name");
        let time_str: String = r.get("check_in_at");
        let time = extract_time(&time_str);
        activities.push(ActivityItem {
            icon: "🟢".to_string(),
            text: format!("Check-in {} → {}", name, room_id),
            time,
            color: "bg-emerald-50".to_string(),
        });
    }

    // Recent check-outs
    let checkouts = sqlx::query(
        "SELECT b.room_id, g.full_name, b.actual_checkout
         FROM bookings b JOIN guests g ON g.id = b.primary_guest_id
         WHERE b.actual_checkout IS NOT NULL
         ORDER BY b.actual_checkout DESC LIMIT ?"
    ).bind(limit).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for r in &checkouts {
        let room_id: String = r.get("room_id");
        let name: String = r.get("full_name");
        let time_str: String = r.get("actual_checkout");
        let time = extract_time(&time_str);
        activities.push(ActivityItem {
            icon: "🔴".to_string(),
            text: format!("Check-out {} — Room {}", name, room_id),
            time,
            color: "bg-red-50".to_string(),
        });
    }

    // Recent housekeeping
    let hk = sqlx::query(
        "SELECT room_id, status, triggered_at FROM housekeeping
         ORDER BY triggered_at DESC LIMIT ?"
    ).bind(limit).fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    for r in &hk {
        let room_id: String = r.get("room_id");
        let status: String = r.get("status");
        let time_str: String = r.get("triggered_at");
        let time = extract_time(&time_str);
        let label = if status == "clean" { "Cleaned" } else { "Needs cleaning" };
        activities.push(ActivityItem {
            icon: "🧹".to_string(),
            text: format!("{} — Room {}", label, room_id),
            time,
            color: "bg-amber-50".to_string(),
        });
    }

    // Sort by time descending and limit
    activities.sort_by(|a, b| b.time.cmp(&a.time));
    activities.truncate(limit as usize);

    Ok(activities)
}

fn extract_time(datetime_str: &str) -> String {
    // Extract HH:MM from ISO datetime or RFC3339
    if let Some(t_pos) = datetime_str.find('T') {
        let time_part = &datetime_str[t_pos + 1..];
        if time_part.len() >= 5 {
            return time_part[..5].to_string();
        }
    }
    // Fallback: try space separator
    if let Some(parts) = datetime_str.split(' ').nth(1) {
        if parts.len() >= 5 {
            return parts[..5].to_string();
        }
    }
    datetime_str.to_string()
}
