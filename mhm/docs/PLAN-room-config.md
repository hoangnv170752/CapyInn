# PLAN: Dynamic Room Configuration

## Overview

Hiện tại hệ thống hardcode 10 phòng qua `seed_rooms()` và room types cố định (`standard`/`deluxe`). Mục tiêu: biến Room Config thành hệ thống quản lý phòng đầy đủ với quy trình rõ ràng.

**Quy trình admin (lần đầu setup):**
1. **Tạo Room Types trước** (ví dụ: "Standard Room", "Deluxe with Balcony")
2. **Tạo Rooms** — chọn room type từ danh sách đã tạo (KHÔNG nhập tự do)

**Pricing model (Option A):**
- `base_price` = giá cơ bản cho `max_guests` người (số người tối thiểu tính giá base)
- `extra_person_fee` = phụ thu mỗi người thêm
- Ví dụ: Standard 300k/2 người, +50k/người → 4 người = 400k

---

## Proposed Changes

### 1. Database — Migration V5

#### [MODIFY] [db.rs](mhm/src-tauri/src/db.rs)

Thêm migration V5 vào `run_migrations()`:

```sql
-- Bảng room_types (admin tạo trước, rooms chọn từ đây)
CREATE TABLE IF NOT EXISTS room_types (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

-- Seed default room types từ rooms hiện tại
INSERT OR IGNORE INTO room_types (id, name, created_at)
  SELECT DISTINCT lower(type), type, datetime('now') FROM rooms;

-- Thêm cột per-person pricing cho rooms
ALTER TABLE rooms ADD COLUMN max_guests INTEGER NOT NULL DEFAULT 2;
ALTER TABLE rooms ADD COLUMN extra_person_fee REAL NOT NULL DEFAULT 0;
```

---

### 2. Backend — Models

#### [MODIFY] [models.rs](mhm/src-tauri/src/models.rs)

- `Room` struct: thêm `max_guests: i32`, `extra_person_fee: f64`
- Thêm `CreateRoomRequest` struct: `id, name, room_type, floor, has_balcony, base_price, max_guests, extra_person_fee`
- Thêm `RoomType` struct: `id, name, created_at`
- Thêm `CreateRoomTypeRequest`: `name`
- Cập nhật `UpdateRoomRequest`: thêm `name, floor, has_balcony, max_guests, extra_person_fee` (all Optional)

---

### 3. Backend — Commands

#### [MODIFY] [commands.rs](mhm/src-tauri/src/commands.rs)

| Command | Mô tả | RBAC |
|---------|-------|------|
| `create_room` | INSERT room mới | admin-only |
| `delete_room` | DELETE room (check vacant + no active booking) | admin-only |
| `get_room_types` | SELECT * FROM room_types | all |
| `create_room_type` | INSERT room type mới | admin-only |
| `delete_room_type` | DELETE room type (check no rooms using it) | admin-only |

Cập nhật:
- `get_rooms` → trả thêm `max_guests`, `extra_person_fee`
- `update_room` → xử lý thêm fields `name, floor, has_balcony, max_guests, extra_person_fee`

#### [MODIFY] [main.rs](mhm/src-tauri/src/main.rs)

Register 5 commands mới: `create_room`, `delete_room`, `get_room_types`, `create_room_type`, `delete_room_type`

---

### 4. Frontend — Room Config UI

#### [MODIFY] [Settings.tsx](mhm/src/pages/Settings.tsx)

**RoomConfigSection redesign:**

Chia thành 2 phần:

**Phần A: Room Types Management** (hiển thị trước)
- Danh sách room types hiện có (badge/chip style)
- Nút "+ Thêm loại phòng" → input tên → save
- Nút xóa (chỉ khi không có room nào đang dùng type đó)

**Phần B: Room List** (CRUD đầy đủ)
- Header "+ Thêm phòng" button
- Form thêm/sửa phòng:
  - Mã phòng (text, ví dụ "1A") — chỉ khi tạo mới
  - Tên phòng (text)
  - Loại phòng (dropdown chọn từ room_types)
  - Tầng (number)
  - Ban công (toggle)
  - Giá cơ bản (number, VNĐ)
  - Số khách tối đa tính giá base (number)
  - Phụ thu/người thêm (number, VNĐ)
- Mỗi phòng: row hiển thị info + nút Sửa + nút Xóa
- Delete có confirm dialog, chặn xóa nếu phòng occupied

**PricingSection update:**
- Room type dropdown lấy từ `get_room_types()` thay vì hardcode

---

## Task Breakdown

### Phase 1: Database + Backend

- [ ] T1.1: Migration V5 trong `db.rs` — thêm `room_types` table + cột `max_guests`, `extra_person_fee`
- [ ] T1.2: Cập nhật models trong `models.rs` — `Room`, `CreateRoomRequest`, `RoomType`, `UpdateRoomRequest`
- [ ] T1.3: Commands trong `commands.rs` — `create_room`, `delete_room`, `get_room_types`, `create_room_type`, `delete_room_type` + cập nhật `get_rooms`, `update_room`
- [ ] T1.4: Register commands trong `main.rs`

### Phase 2: Frontend

- [ ] T2.1: `RoomConfigSection` redesign — Room Types management + Room list CRUD
- [ ] T2.2: `PricingSection` — dynamic room type dropdown
- [ ] T2.3: Cập nhật `Room` interface TypeScript

### Phase 3: Polish

- [ ] T3.1: Cập nhật seed data mặc định cho `max_guests` và `extra_person_fee`

---

## Verification Plan

### Build Verification
```bash
cd ./mhm/src-tauri && cargo build
cd ./mhm && npm run build
```

### Existing Tests (đảm bảo không break)
```bash
cd ./mhm && npx vitest run tests/e2e/08-settings.test.tsx
cd ./mhm/src-tauri && cargo test
```

### Manual Verification (anh test trên app)
1. Mở Settings → Room Config → thấy phần "Loại phòng" và "Danh sách phòng"
2. Tạo loại phòng mới (VD: "Deluxe with Balcony") → xuất hiện trong danh sách
3. Thêm phòng mới, chọn loại từ dropdown → phòng xuất hiện
4. Sửa phòng (đổi giá, sức chứa, phụ thu) → save → hiển thị đúng
5. Xóa phòng vacant → OK. Thử xóa phòng occupied → lỗi
6. Kiểm tra trang Rooms chính → phòng mới hiện đúng
7. Kiểm tra PricingSection → dropdown loại phòng hiển thị types mới
