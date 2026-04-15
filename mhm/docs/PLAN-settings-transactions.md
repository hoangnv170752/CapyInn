# PLAN: Settings Persistence + Analytics Transactions (P0)

## Mục tiêu

Sửa 2 vấn đề P0:
1. **Settings không lưu** — Hotel Info, Check-in Rules chỉ lưu localStorage, cần persist vào DB
2. **Analytics trống** — `get_analytics` query `transactions` table nhưng table luôn trống vì check-in không tạo transaction

---

## Phân tích hiện trạng

### Transactions

| Flow | Hiện tại | Vấn đề |
|------|----------|--------|
| `check_in` | Tạo transaction khi `paid > 0` | Frontend gửi `paid_amount = 0` → không tạo transaction nào |
| `check_out` | Tạo transaction khi `final_paid > already_paid` | Nếu không truyền `final_paid`, không tạo transaction |
| `get_analytics` | Query `SUM(amount) FROM transactions WHERE type='payment'` | Luôn = 0 vì không có transactions |

**Root cause:** `check_in` PHẢI tạo transaction `type='charge'` cho `total_price` bất kể khách trả tiền hay chưa. `get_analytics` cần query cả `charge` transaction (= doanh thu phòng) thay vì chỉ `payment`.

### Settings

| Component | Hiện tại | Cần |
|-----------|----------|-----|
| Hotel Info | `localStorage` only | DB `settings` table |
| Check-in Rules | `localStorage` only | DB `settings` table |
| Room Config | Hardcoded | Có `update_room` command rồi ✅ |
| Appearance | Đã xóa dark mode | Chỉ còn language → `localStorage` OK |

---

## Proposed Changes

### 1. Backend: Transaction trên Check-in (Rust)

#### [MODIFY] [commands.rs](mhm/src-tauri/src/commands.rs)

**Trong `check_in` function (line ~150):**

Thêm ALWAYS-CREATE transaction cho total_price:

```rust
// Always create charge transaction for room revenue
let charge_id = uuid::Uuid::new_v4().to_string();
sqlx::query(
    "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
     VALUES (?, ?, ?, 'charge', 'Tiền phòng', ?)"
)
.bind(&charge_id).bind(&booking_id).bind(total_price).bind(now.to_rfc3339())
.execute(&state.db).await.map_err(|e| e.to_string())?;
```

**Trong `check_out` function (line ~270):**

Thêm charge transaction cho toàn bộ checkout amount nếu chưa có:

```rust
// Always create checkout charge if no final payment specified
if req.final_paid.is_none() {
    let txn_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
         VALUES (?, ?, ?, 'payment', 'Thanh toán khi check-out', ?)"
    )
    .bind(&txn_id).bind(&req.booking_id).bind(total).bind(now.to_rfc3339())
    .execute(&state.db).await.map_err(|e| e.to_string())?;
}
```

**Trong `get_analytics` function (line ~693):**

Đổi revenue query từ `type='payment'` sang `type='charge'` (vì charge = doanh thu phòng thực tế):

```sql
-- Trước
SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE type = 'payment' AND ...
-- Sau
SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE type = 'charge' AND ...
```

Đổi daily revenue query tương tự.

---

### 2. Backend: Settings Persistence (Rust)

#### [MODIFY] [db.rs](mhm/src-tauri/src/db.rs)

Thêm `settings` table trong `run_migrations`:

```sql
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
)
```

#### [MODIFY] [commands.rs](mhm/src-tauri/src/commands.rs)

Thêm 2 commands mới:

```rust
#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, key: String, value: String) -> Result<(), String>

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>, key: String) -> Result<Option<String>, String>
```

#### [MODIFY] [lib.rs](mhm/src-tauri/src/lib.rs)

Register 2 commands mới: `save_settings`, `get_settings`

---

### 3. Frontend: Wire Settings to Backend

#### [MODIFY] [Settings.tsx](mhm/src/pages/Settings.tsx)

- `HotelInfoSection`: Load từ `get_settings("hotel_info")` on mount, save bằng `save_settings("hotel_info", JSON.stringify(data))`
- `CheckinRulesSection`: Load từ `get_settings("checkin_rules")` on mount, save bằng `save_settings("checkin_rules", JSON.stringify(data))`
- Remove `localStorage` calls, thay bằng `invoke`

---

## Backfill: Tạo transactions cho bookings hiện có

Cần chạy 1 lần SQL để tạo charge transactions cho 6 bookings đang có:

```sql
INSERT INTO transactions (id, booking_id, amount, type, note, created_at)
SELECT
    lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(6))),
    id, total_price, 'charge', 'Tiền phòng (backfill)', created_at
FROM bookings WHERE id NOT IN (SELECT booking_id FROM transactions WHERE type = 'charge');
```

---

## Verification Plan

### Test thủ công trong Tauri window

1. **Transaction tạo đúng:**
   - Mở Tauri app → Dashboard → Check-in khách mới
   - Chạy `sqlite3 ~/MHM/mhm.db "SELECT * FROM transactions"` → phải có 1 row `type='charge'`
   - Vào Analytics → revenue phải > 0

2. **Settings persist:**
   - Mở Settings → Hotel Info → Đổi tên → Lưu
   - Chuyển tab khác → Quay lại Settings → Tên còn nguyên
   - Restart app (`npm run tauri dev` lại) → Tên VẪN còn nguyên

3. **Check-in Rules persist:**
   - Mở Settings → Check-in Rules → Đổi giờ check-in → Lưu
   - Chuyển tab → quay lại → giờ còn nguyên
   - Restart app → giờ VẪN còn nguyên

4. **Analytics hiển thị:**
   - Vào Analytics → có biểu đồ revenue, occupancy rate > 0, top rooms hiện data
