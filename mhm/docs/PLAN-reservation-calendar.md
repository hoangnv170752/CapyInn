# PLAN: Reservation + Calendar Block System (Industry-Standard)

> Created: 2026-03-16 | Agent: `project-planner` | Status: **AWAITING REVIEW**

---

## Mục tiêu

Implement tính năng đặt phòng trước (reservation) với hệ thống Calendar Block để:
1. Nhận đặt phòng + tiền cọc cho ngày tương lai
2. **Ngăn overbooking** — kiểm tra xung đột ngày khi check-in hoặc tạo reservation mới
3. Hiển thị trực quan trên timeline và dashboard

---

## User Review Required

> [!IMPORTANT]
> **Giả định chưa được confirm (cần anh xác nhận):**
> 1. **Chính sách cọc khi hủy:** Em giả định **mất cọc** khi khách hủy (ghi nhận deposit là revenue). Anh muốn khác?
> 2. **No-show:** Em giả định phải **manual cancel** — không auto-cancel. OK?
> 3. **Thông tin tối thiểu khi đặt:** Em giả định chỉ cần **tên + SĐT**. CCCD scan lúc check-in thật. Đúng không?
> 4. **Overbooking policy:** Từ brainstorm anh đã confirm = **KHÔNG cho phép overbooking**. Em block cứng.

---

## Proposed Changes

### Tổng quan kiến trúc

```
┌─────────────────────────────────────────────────────────────────┐
│  room_calendar (MỚI)                                             │
│  ├── (room_id, date) = PRIMARY KEY                              │
│  ├── booking_id → links tới bookings table                      │
│  ├── status: 'booked' | 'occupied' | 'blocked' | 'maintenance' │
│  └── 10 rooms × 365 days = ~3,650 rows/năm                     │
└───────────────────────────────┬─────────────────────────────────┘
                                │
    ┌───────────────────────────┼───────────────────────────┐
    │                           │                           │
    ▼                           ▼                           ▼
create_reservation()     check_in()                 cancel_reservation()
→ INSERT calendar rows   → CHECK calendar overlap   → DELETE calendar rows
→ INSERT booking(booked) → UPDATE calendar→occupied → UPDATE booking(cancelled)
→ INSERT txn(deposit)    → UPDATE room→occupied     → UPDATE room→vacant
```

---

### Phase 1: Database Migration (v6)

#### [MODIFY] [db.rs](mhm/src-tauri/src/db.rs)

Thêm migration v6 tạo bảng `room_calendar` và thêm reservation fields vào `bookings`:

```sql
-- Bảng room_calendar: mỗi row = 1 ngày bị block cho 1 phòng
CREATE TABLE IF NOT EXISTS room_calendar (
    room_id    TEXT NOT NULL REFERENCES rooms(id),
    date       TEXT NOT NULL,              -- YYYY-MM-DD
    booking_id TEXT REFERENCES bookings(id) ON DELETE CASCADE,
    status     TEXT NOT NULL DEFAULT 'booked',  
    -- 'booked' | 'occupied' | 'blocked' | 'maintenance'
    PRIMARY KEY (room_id, date)
);

CREATE INDEX idx_calendar_booking ON room_calendar(booking_id);
CREATE INDEX idx_calendar_status ON room_calendar(room_id, status);

-- Thêm fields cho Reservation vào bookings
ALTER TABLE bookings ADD COLUMN booking_type TEXT DEFAULT 'walk-in';
-- 'walk-in' | 'reservation'
ALTER TABLE bookings ADD COLUMN deposit_amount REAL DEFAULT 0;
ALTER TABLE bookings ADD COLUMN guest_phone TEXT;
ALTER TABLE bookings ADD COLUMN scheduled_checkin TEXT;
-- Ngày hẹn đến (YYYY-MM-DD), dùng cho reservation
ALTER TABLE bookings ADD COLUMN scheduled_checkout TEXT;
-- Ngày hẹn đi (YYYY-MM-DD)
```

---

### Phase 2: Backend — Models & Availability Logic

#### [MODIFY] [models.rs](mhm/src-tauri/src/models.rs)

Thêm DTOs mới:

```rust
// Request tạo reservation
pub struct CreateReservationRequest {
    pub room_id: String,
    pub guest_name: String,
    pub guest_phone: Option<String>,
    pub guest_doc_number: Option<String>,   // optional, scan lúc check-in
    pub check_in_date: String,              // YYYY-MM-DD
    pub check_out_date: String,             // YYYY-MM-DD
    pub nights: i32,
    pub deposit_amount: Option<f64>,
    pub source: Option<String>,             // phone/agoda/booking.com/zalo
    pub notes: Option<String>,
}

// Response availability check
pub struct AvailabilityResult {
    pub available: bool,
    pub conflicts: Vec<CalendarConflict>,
    pub max_nights: Option<i32>,           // Tối đa bao nhiêu đêm
}

pub struct CalendarConflict {
    pub date: String,
    pub status: String,
    pub guest_name: Option<String>,
    pub booking_id: String,
}

// Calendar entry (for frontend display)
pub struct CalendarEntry {
    pub room_id: String,
    pub date: String,
    pub booking_id: Option<String>,
    pub status: String,
}
```

Cập nhật `status::booking` thêm:
```rust
pub const BOOKED: &str = "booked";      // đã có nhưng chưa dùng
pub const CANCELLED: &str = "cancelled";
pub const NO_SHOW: &str = "no_show";
```

---

### Phase 3: Backend — 6 Commands mới

#### [MODIFY] [commands.rs](mhm/src-tauri/src/commands.rs)

**6 commands mới + sửa 2 commands cũ:**

| # | Command | Mô tả |
|---|---------|-------|
| 1 | `check_availability` | Query `room_calendar` xem date range có conflict không |
| 2 | `create_reservation` | Tạo booking (status=booked) + insert calendar rows + deposit |
| 3 | `confirm_reservation` | Convert reservation → active booking (check-in thật) |
| 4 | `cancel_reservation` | Cancel booking + delete calendar rows + xử lý deposit |
| 5 | `get_room_calendar` | Lấy calendar entries cho 1 phòng trong date range |
| 6 | `get_rooms_availability` | Lấy tất cả phòng + upcoming reservations (cho dashboard) |
| 7 | **Sửa `check_in()`** | Thêm calendar overlap check trước khi check-in walk-in |
| 8 | **Sửa `check_out()`** | Xóa calendar rows khi checkout |

**Chi tiết logic từng command:**

**1. `check_availability(room_id, from_date, to_date)`:**
```sql
SELECT date, status, booking_id 
FROM room_calendar 
WHERE room_id = ? AND date >= ? AND date < ?
```
- Trả về `AvailabilityResult { available, conflicts, max_nights }`
- `max_nights` = số ngày tối đa trước conflict đầu tiên

**2. `create_reservation(req)`:**
- Gọi `check_availability()` trước — reject nếu conflict
- INSERT booking (status="booked", booking_type="reservation")
- INSERT guest (full_name + phone, doc_number optional)
- INSERT vào `room_calendar` cho mỗi ngày: `(room_id, date, booking_id, "booked")`
- Nếu có deposit: INSERT transaction (type="deposit")
- **KHÔNG** đổi room.status (phòng vẫn vacant hiện tại)

**3. `confirm_reservation(booking_id)`:**
- Load booking (phải status="booked")
- UPDATE booking: status → "active", check_in_at = NOW()
- UPDATE room_calendar: status → "occupied"
- UPDATE room: status → "occupied"
- Deposit tính vào paid_amount

**4. `cancel_reservation(booking_id)`:**
- UPDATE booking: status → "cancelled"
- DELETE FROM room_calendar WHERE booking_id = ?
- **Deposit giữ lại** (INSERT transaction type="cancellation_fee" hoặc tùy policy)

**5. `get_room_calendar(room_id, from, to)`:**
- SELECT tất cả calendar entries cho room trong date range
- Frontend dùng để render calendar view

**6. `get_rooms_availability(from, to)`:**
- Cho mỗi room: lấy current booking + upcoming reservations
- Dashboard dùng để hiện badge ⚡ "Đặt 26/03"

**7. Sửa `check_in()` (CRITICAL):**
```rust
// THAY ĐỔI: Không chỉ check room.status == "vacant"
// Mà phải check calendar overlap cho date range mới

let conflicts = check_availability_internal(
    &tx, &req.room_id, &checkin_date, &checkout_date
).await?;

if !conflicts.is_empty() {
    return Err(format!(
        "Phòng {} đã có đặt phòng từ ngày {}. Tối đa {} đêm.",
        req.room_id, conflicts[0].date, max_nights
    ));
}

// Sau khi check-in OK → insert calendar rows (status="occupied")
```

**8. Sửa `check_out()`:**
- DELETE FROM room_calendar WHERE booking_id = ? (xóa occupied days)

---

### Phase 4: Frontend — Reservation Form

#### [MODIFY] [Reservations.tsx](mhm/src/pages/Reservations.tsx)

Thêm chức năng:
- Nút **"+ Đặt phòng"** mở ReservationSheet 
- Timeline hiện bar màu khác cho `booked` vs `active`:
  - 🔵 `booked` = xanh dương (reservation tương lai)
  - 🟢 `active` = xanh lá (đang ở)  
  - 🟡 `checked_out` = vàng
- Click vào bar `booked` → hiện options: Confirm Check-in / Cancel / Edit

#### [NEW] [ReservationSheet.tsx](mhm/src/components/ReservationSheet.tsx)

Form tạo reservation mới:
```
┌────────────────────────────────────────┐
│  📅 Đặt phòng mới                      │
│                                         │
│  Phòng:     [Dropdown - chỉ hiện phòng │
│              available trong date range]│
│                                         │
│  Ngày đến:  [Date picker]              │
│  Ngày đi:   [Date picker]              │
│  Số đêm:    [Auto-calculated]          │
│                                         │
│  Họ tên:    [________________]          │
│  SĐT:      [________________]          │
│  CCCD:      [________________] (tùy chọn)│
│                                         │
│  Nguồn:    [phone ▾] agoda/booking.com  │
│  Tiền cọc: [____________] VNĐ          │
│  Ghi chú:  [________________]          │
│                                         │
│  ⚠️ Phòng 3A available 16/03 → 25/03   │
│     (có đặt phòng khác từ 26/03)       │
│                                         │
│  [Hủy]              [Đặt phòng]        │
└────────────────────────────────────────┘
```

Key UX behaviors:
- Khi chọn phòng + ngày → gọi `check_availability()` realtime
- Nếu conflict → hiện cảnh báo + gợi ý max_nights
- Date range picker chỉ cho chọn ngày available

---

### Phase 5: Frontend — Dashboard Badge

#### [MODIFY] [Dashboard.tsx](mhm/src/pages/Dashboard.tsx)

#### [MODIFY] [RoomCard.tsx](mhm/src/components/RoomCard.tsx)

- Gọi `get_rooms_availability()` thay vì `get_rooms()`
- RoomCard hiện thêm badge nhỏ nếu có reservation sắp tới:
```
┌──────────────┐
│   3A         │
│   🟢 Trống   │
│   ⚡ Đặt 26/03│  ← badge cho reservation tương lai
└──────────────┘
```

---

### Phase 6: Sửa Check-in Flow

#### [MODIFY] [CheckinSheet.tsx](mhm/src/components/CheckinSheet.tsx)

- Khi chọn phòng + số đêm → gọi `check_availability()` 
- Nếu conflict → hiện warning inline:
```
⚠️ Phòng 3A đã có đặt phòng từ 26/03 (Nguyễn Văn A, 2 đêm).
   Tối đa 10 đêm (check-out trước 26/03).
   [Điều chỉnh] [Chọn phòng khác]
```
- Hỗ trợ check-in từ reservation: khi click "Confirm Check-in" trên reservation bar → mở CheckinSheet với thông tin pre-filled

---

### Phase 7: Lib Registration + Event Wiring

#### [MODIFY] [lib.rs](mhm/src-tauri/src/lib.rs)

Register 6 commands mới vào `invoke_handler`:
```rust
commands::check_availability,
commands::create_reservation,
commands::confirm_reservation,
commands::cancel_reservation,
commands::get_room_calendar,
commands::get_rooms_availability,
```

---

## Impacted Queries Summary

Các query hiện có cần **KHÔNG thay đổi** nhờ calendar approach:
- `get_analytics()` — dùng bookings table, không ảnh hưởng
- `get_revenue_stats()` — dùng transactions table
- `run_night_audit()` — dùng bookings + transactions
- `get_all_bookings()` — cần thêm filter cho `booked` status
- `export_csv()` — cần include reservation bookings

---

## Implementation Order

```
Phase 1 (DB)      → đảm bảo migration chạy clean
Phase 2 (Models)  → compile check
Phase 3 (Backend) → tất cả commands hoạt động
Phase 7 (Lib)     → register commands  
Phase 4 (Frontend Reservation) → tạo + hiển thị reservation
Phase 5 (Dashboard) → badge hiển thị
Phase 6 (Check-in) → conflict check khi walk-in
```

---

## Verification Plan

### Automated Tests

**Existing test cần update:**
```bash
npm test -- tests/e2e/09-reservations.test.tsx   # hiện chỉ test read-only timeline
npm test -- tests/e2e/03-checkin.test.tsx         # cần thêm test overbooking reject
```

**Test mới cần viết:**
- `tests/e2e/13-reservation-flow.test.tsx`:
  - Tạo reservation → verify booking status = "booked"
  - Tạo reservation trùng ngày → verify reject  
  - Walk-in check-in trùng ngày reservation → verify reject + error message
  - Cancel reservation → verify calendar rows deleted
  - Confirm reservation (check-in) → verify status = "active"

**Rust unit tests:**
```bash
cd mhm/src-tauri && cargo test
```
- Thêm test cho `check_availability()` logic trong `commands.rs`

### Manual Verification

Anh test trực tiếp trên app:

1. **Happy path:** Tạo reservation phòng 3A cho 10 ngày sau → Dashboard hiện badge ⚡ → Confirm check-in khi khách đến
2. **Overbooking prevention:** Thử check-in walk-in phòng 3A với số đêm trùng ngày reservation → verify app hiện cảnh báo
3. **Cancel flow:** Tạo reservation → Cancel → verify phòng available lại trên timeline
4. **Timeline display:** Verify Reservations page hiện bar 🔵 cho reservation tương lai

---

## Estimated Effort

| Phase | Effort | Files |
|-------|--------|-------|
| 1. DB Migration | 0.5 ngày | `db.rs` |
| 2. Models | 0.5 ngày | `models.rs` |
| 3. Backend Commands | 2 ngày | `commands.rs` |
| 4. Reservation UI | 1.5 ngày | `Reservations.tsx`, `ReservationSheet.tsx` (NEW) |
| 5. Dashboard Badge | 0.5 ngày | `Dashboard.tsx`, `RoomCard.tsx` |
| 6. Check-in Update | 0.5 ngày | `CheckinSheet.tsx`, `commands.rs` |
| 7. Lib + Tests | 0.5 ngày | `lib.rs`, tests |
| **Total** | **~5-6 ngày** | |

---

*Plan v1.0 — Reservation Calendar Block System*
