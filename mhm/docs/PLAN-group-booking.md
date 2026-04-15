# Group Booking Implementation Plan

Thêm tính năng Group Booking cho Hotel Manager, cho phép check-in đoàn khách lớn (10-30+ người) vào nhiều phòng cùng lúc. Bao gồm group lifecycle management, partial checkout, group invoice với dịch vụ kèm, và auto/manual room assignment.

> [!IMPORTANT]
> Feature này thay đổi DB schema (migration V9), thêm module backend mới, và tạo UI components mới. Không breaking change với flow check-in hiện tại.

---

## Phase 1: Database Migration (V9)

### [MODIFY] `src-tauri/src/db.rs`

Thêm migration V9 sau block V8 (line ~397). Tạo 2 bảng mới + thêm cột vào `bookings`:

```sql
-- Bảng 1: booking_groups
CREATE TABLE IF NOT EXISTS booking_groups (
    id                TEXT PRIMARY KEY,
    group_name        TEXT NOT NULL,
    master_booking_id TEXT,
    organizer_name    TEXT NOT NULL,
    organizer_phone   TEXT,
    total_rooms       INTEGER NOT NULL,
    status            TEXT NOT NULL DEFAULT 'active',
    notes             TEXT,
    created_by        TEXT,
    created_at        TEXT NOT NULL
);

-- Bảng 2: group_services (giặt ủi, xe máy, tour...)
CREATE TABLE IF NOT EXISTS group_services (
    id          TEXT PRIMARY KEY,
    group_id    TEXT NOT NULL REFERENCES booking_groups(id),
    booking_id  TEXT REFERENCES bookings(id),
    name        TEXT NOT NULL,
    quantity    INTEGER NOT NULL DEFAULT 1,
    unit_price  REAL NOT NULL,
    total_price REAL NOT NULL,
    note        TEXT,
    created_by  TEXT,
    created_at  TEXT NOT NULL
);

-- Thêm cột vào bookings
ALTER TABLE bookings ADD COLUMN group_id TEXT REFERENCES booking_groups(id);
ALTER TABLE bookings ADD COLUMN is_master_room INTEGER DEFAULT 0;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bookings_group ON bookings(group_id);
CREATE INDEX IF NOT EXISTS idx_group_services_group ON group_services(group_id);
```

**Group status lifecycle:** `active` → `partial_checkout` → `completed`

---

## Phase 2: Backend — Models & Commands

### [MODIFY] `src-tauri/src/models.rs`

Thêm structs:

```rust
// ── Group Booking DTOs ──

pub struct BookingGroup {
    pub id: String,
    pub group_name: String,
    pub master_booking_id: Option<String>,
    pub organizer_name: String,
    pub organizer_phone: Option<String>,
    pub total_rooms: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

pub struct GroupService {
    pub id: String,
    pub group_id: String,
    pub booking_id: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub note: Option<String>,
    pub created_at: String,
}

pub struct GroupCheckinRequest {
    pub group_name: String,
    pub organizer_name: String,
    pub organizer_phone: Option<String>,
    pub room_ids: Vec<String>,           // Danh sách phòng
    pub master_room_id: String,          // Phòng đại diện
    pub guests_per_room: HashMap<String, Vec<CreateGuestRequest>>,
    pub nights: i32,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub paid_amount: Option<f64>,
}

pub struct GroupCheckoutRequest {
    pub group_id: String,
    pub booking_ids: Vec<String>,        // Subset để checkout
    pub final_paid: Option<f64>,
}

pub struct AddGroupServiceRequest {
    pub group_id: String,
    pub booking_id: Option<String>,      // NULL = tính cho cả đoàn
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub note: Option<String>,
}

pub struct GroupDetailResponse {
    pub group: BookingGroup,
    pub bookings: Vec<BookingWithGuest>,
    pub services: Vec<GroupService>,
    pub total_room_cost: f64,
    pub total_service_cost: f64,
    pub grand_total: f64,
    pub paid_amount: f64,
}

pub struct AutoAssignResult {
    pub assignments: Vec<RoomAssignment>,
}

pub struct RoomAssignment {
    pub room: Room,
    pub floor: i32,
}
```

---

### [NEW] `src-tauri/src/commands/groups.rs`

Module mới cho group booking commands. 8 Tauri commands:

| Command | Mô tả |
|---------|--------|
| `group_checkin` | Check-in đoàn: tạo group + N bookings trong 1 transaction |
| `group_checkout` | Checkout subset phòng, cập nhật group status |
| `get_group_detail` | Lấy chi tiết group (bookings, services, totals) |
| `get_all_groups` | Danh sách tất cả groups (filter by status) |
| `add_group_service` | Thêm dịch vụ (giặt ủi, xe máy, tour) |
| `remove_group_service` | Xóa dịch vụ |
| `auto_assign_rooms` | Tự động chọn phòng (cùng tầng, gần nhau) |
| `generate_group_invoice` | Xuất hóa đơn tổng cho group |

#### `group_checkin` logic:
1. Validate: tất cả room_ids phải vacant
2. Check calendar overlap cho mỗi phòng
3. Tạo `booking_groups` record
4. Loop qua room_ids:
   - Tạo guests (from `guests_per_room` hoặc empty)
   - Tạo booking với `group_id` và `is_master_room` flag
   - Tạo `booking_guests` links
   - Block `room_calendar` entries
   - Update room status → occupied
5. Set `master_booking_id` on group
6. Tạo charge transaction
7. Emit `db-updated`

#### `group_checkout` logic:
1. Fetch group + selected bookings
2. Loop qua `booking_ids`:
   - Run same check_out logic (update booking, room → cleaning, housekeeping, clear calendar)
3. Kiểm tra phòng master:
   - Nếu master checkout → tự động chọn phòng active đầu tiên làm master mới
4. Update group status:
   - Tất cả checkout → `completed`
   - Còn phòng active → `partial_checkout`
5. Nếu `final_paid` → record transaction

#### `auto_assign_rooms` logic:
1. Input: `room_count`, `room_type` (optional)
2. Lọc vacant rooms
3. Sort by floor → group cùng tầng
4. Greedy fill: ưu tiên tầng có nhiều phòng trống nhất
5. Return danh sách phòng đề xuất

---

### [MODIFY] `src-tauri/src/commands/mod.rs`

Thêm:
```rust
pub mod groups;
pub use groups::{do_get_group_detail, do_group_checkin};
```

### [MODIFY] `src-tauri/src/lib.rs`

Thêm vào `generate_handler![]`:
```rust
// Group Booking
commands::groups::group_checkin,
commands::groups::group_checkout,
commands::groups::get_group_detail,
commands::groups::get_all_groups,
commands::groups::add_group_service,
commands::groups::remove_group_service,
commands::groups::auto_assign_rooms,
commands::groups::generate_group_invoice,
```

---

## Phase 3: Frontend — Group Check-in UI

### [NEW] `src/components/GroupCheckinSheet.tsx`

Multi-step wizard dạng Sheet (giống `CheckinSheet` nhưng rộng hơn):

```
Step 1: Thông tin đoàn
├── Tên đoàn (text)
├── Tên trưởng đoàn (text)
├── SĐT trưởng đoàn (text)
├── Số phòng cần (number)
├── Loại phòng (select: all/standard/deluxe)
├── Số đêm (number)
└── Nguồn (walk-in/agoda/booking.com/phone)

Step 2: Chọn phòng
├── [Auto-assign ✨] → gọi auto_assign_rooms → hiển thị result
├── [Manual-assign 🖐️] → room grid/list cho chọn tay
├── Hiển thị danh sách phòng đã chọn
└── Chọn phòng đại diện (radio button)

Step 3: Thông tin khách (per room)
├── Accordion/Tab cho mỗi phòng
├── Mỗi phòng: scan CCCD hoặc nhập tay
├── Phòng đại diện bắt buộc có khách chính
└── Các phòng khác có thể bỏ trống (điền sau)

Step 4: Xác nhận & Thanh toán
├── Summary: N phòng × giá = tổng tiền
├── Trả trước (number)
├── Ghi chú
└── [Hoàn tất Group Check-in]
```

### [MODIFY] `src/stores/useHotelStore.ts`

Thêm state + actions:
```typescript
isGroupCheckinOpen: boolean;
setGroupCheckinOpen: (open: boolean) => void;
groupCheckIn: (req: GroupCheckinRequest) => Promise<void>;
```

### [MODIFY] `src/pages/Dashboard.tsx`

Thêm button "Đoàn mới" cạnh button "Khách mới" hiện tại.

---

## Phase 4: Frontend — Group Management UI

### [NEW] `src/pages/GroupManagement.tsx`

Hoặc thêm tab/section trong trang hiện tại. Gồm:

**Danh sách groups:**
- Table: Tên đoàn | Trưởng đoàn | Số phòng | Status | Ngày check-in | Actions
- Filter: active / partial_checkout / completed

**Chi tiết group (drawer/dialog):**
- Thông tin đoàn
- Danh sách phòng (checkbox để select checkout)
- Dịch vụ kèm (CRUD table)
- Tổng tiền (rooms + services)
- Actions: Checkout selected | Add service | Generate invoice

### [NEW] `src/components/GroupInvoice.tsx`

Component hiển thị/in hóa đơn đoàn (layout theo mẫu trong brainstorm deep dive).

---

## Phase 5: Group Invoice Backend

### Logic trong `groups.rs`

`generate_group_invoice` logic:
1. Fetch group + all bookings + all services
2. Tính tổng phòng: sum(booking.total_price)
3. Tính tổng dịch vụ: sum(service.total_price)
4. Tính đã trả: sum(paid_amount across bookings)
5. Return `GroupInvoiceData` struct

### [MODIFY] `src-tauri/src/models.rs`

```rust
pub struct GroupInvoiceData {
    pub group: BookingGroup,
    pub rooms: Vec<GroupInvoiceRoomLine>,
    pub services: Vec<GroupService>,
    pub subtotal_rooms: f64,
    pub subtotal_services: f64,
    pub grand_total: f64,
    pub paid_amount: f64,
    pub balance_due: f64,
    pub hotel_name: String,
    pub hotel_address: String,
    pub hotel_phone: String,
}

pub struct GroupInvoiceRoomLine {
    pub room_name: String,
    pub room_type: String,
    pub nights: i32,
    pub price_per_night: f64,
    pub total: f64,
    pub guest_name: String,
}
```

---

## Tóm tắt Files

| Action | File | Mô tả |
|--------|------|--------|
| MODIFY | `src-tauri/src/db.rs` | Migration V9 |
| MODIFY | `src-tauri/src/models.rs` | 8+ structs mới |
| NEW | `src-tauri/src/commands/groups.rs` | 8 commands |
| MODIFY | `src-tauri/src/commands/mod.rs` | Register module |
| MODIFY | `src-tauri/src/lib.rs` | Register commands |
| NEW | `src/components/GroupCheckinSheet.tsx` | Multi-step wizard |
| NEW | `src/components/GroupInvoice.tsx` | Invoice display |
| NEW | `src/pages/GroupManagement.tsx` | Group list + detail |
| MODIFY | `src/stores/useHotelStore.ts` | Group actions |
| MODIFY | `src/pages/Dashboard.tsx` | "Đoàn mới" button |
| MODIFY | `src/types/index.ts` | TypeScript types |

---

## Verification Plan

### Automated Tests

Thêm test file `tests/e2e/13-group-booking.test.tsx` theo pattern hiện có (`03-checkin.test.tsx`):

```bash
cd ./mhm && npx vitest run tests/e2e/13-group-booking.test.tsx
```

Test cases:
1. Group check-in tạo đúng N bookings với group_id
2. Auto-assign trả về phòng cùng tầng
3. Partial checkout cập nhật group status → `partial_checkout`
4. Full checkout → group status = `completed`
5. Master auto-transfer khi master checkout
6. Add/remove group service
7. Group invoice tính đúng tổng (rooms + services)

### Build Verification

```bash
cd ./mhm && cargo build 2>&1 | head -20
```

### Manual Verification

Anh cần test trực tiếp trên app:
1. Mở app → Dashboard → click "Đoàn mới"
2. Nhập thông tin đoàn, chọn 3 phòng, check-in
3. Vào Group Management → verify danh sách phòng
4. Thêm dịch vụ giặt ủi cho 1 phòng
5. Checkout 2 phòng → verify status "partial_checkout"
6. Generate invoice → verify tổng tiền đúng
7. Checkout phòng cuối → verify status "completed"

---

## Thứ tự Implementation

```
Phase 1 (DB) → Phase 2 (Backend) → cargo build ✅
→ Phase 3 (Group Checkin UI) → Phase 4 (Group Management UI)
→ Phase 5 (Invoice) → Tests → Manual QA
```

Estimated: **4-5 ngày** (có thể chia nhỏ ship từng phase).
