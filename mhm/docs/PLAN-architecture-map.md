# MHM — Architecture Map & System Diagrams 🏨

> Generated: 2026-03-16 | Based on full codebase analysis

---

## 1. Kiến trúc tổng quan (System Architecture)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        MHM Desktop Application                         │
│                           (Tauri 2.0 Shell)                            │
├──────────────────────────────┬──────────────────────────────────────────┤
│    FRONTEND (WebView)        │          BACKEND (Rust)                  │
│                              │                                          │
│  ┌────────────────────────┐  │  ┌─────────────────────────────────────┐ │
│  │  React 19 + TypeScript │  │  │           lib.rs (Entry)            │ │
│  │  ├── 12 Pages          │  │  │  ├── Setup Tauri plugins            │ │
│  │  ├── 3 Components      │◄─IPC─►  ├── Init DB + Migrations          │ │
│  │  ├── 2 Zustand Stores  │  │  │  ├── Start File Watcher            │ │
│  │  └── shadcn/ui         │  │  │  └── Register 40+ Commands          │ │
│  └────────────────────────┘  │  └──────────────┬──────────────────────┘ │
│                              │                  │                       │
│  ┌────────────────────────┐  │  ┌───────────────▼───────────────────┐  │
│  │  Zustand Stores        │  │  │         commands.rs               │  │
│  │  ├── useHotelStore     │  │  │  ├── Room CRUD                    │  │
│  │  │   (rooms, bookings, │  │  │  ├── Guest Management             │  │
│  │  │    housekeeping)     │  │  │  ├── Check-in / Check-out         │  │
│  │  └── useAuthStore      │  │  │  ├── Pricing Engine                │  │
│  │      (login, session)   │  │  │  ├── Folio / Billing              │  │
│  └────────────────────────┘  │  │  ├── Night Audit                   │  │
│                              │  │  ├── Analytics & Stats             │  │
│                              │  │  ├── Auth & RBAC                   │  │
│                              │  │  └── Backup & Export               │  │
│                              │  └───────────────┬───────────────────┘  │
│                              │                  │                       │
│                              │  ┌───────────────▼───────────────────┐  │
│                              │  │    SQLite (via sqlx)              │  │
│                              │  │    ~/MHM/mhm.db                  │  │
│                              │  └──────────────────────────────────┘  │
└──────────────────────────────┴──────────────────────────────────────────┘
         ▲                                        ▲
         │ Event: "ocr-result"                     │ File System
         │ Event: "db-updated"                     │
         │                                         │
    ┌────┴──────────────┐              ┌───────────┴──────────┐
    │  Tauri Events      │              │  ~/MHM/Scans/        │
    │  (Real-time IPC)   │              │  (Canon LiDE 300)    │
    └───────────────────┘              └──────────────────────┘
```

---

## 2. Module Dependency Map (Backend Rust)

```
                         main.rs
                            │
                         lib.rs
                       ┌────┴────────────────────────────┐
                       │                                  │
                  commands.rs                        watcher.rs
               ┌───┬───┬───┐                             │
               │   │   │   │                          ocr.rs
           models.rs │  │ pricing.rs                     │
                   db.rs │                        PaddleOCR v5
                         │                      (Metal GPU)
                    sqlx::Pool<Sqlite>
                         │
                     mhm.db
```

### File Inventory

| File | Lines | Vai trò |
|------|-------|---------|
| `lib.rs` | 96 | App bootstrap, plugin setup, command registration |
| `commands.rs` | 1821 | **40+ IPC commands** — toàn bộ business logic |
| `models.rs` | 321 | 20+ data structs (Room, Guest, Booking, DTOs) |
| `db.rs` | 402 | DB init, versioned migrations (v1→v10), seed data |
| `pricing.rs` | 524 | VN pricing engine (hourly/overnight/daily/nightly) |
| `watcher.rs` | 101 | File watcher cho `~/MHM/Scans/` |
| `ocr.rs` | 148 | PaddleOCR v5 integration, CCCD parser |

---

## 3. State Machine Diagrams

### 3.1 Room Status State Machine

Đây là vòng đời trạng thái của phòng:

```
                         ┌──── seed_rooms() ────┐
                         ▼                      │
                    ╔═══════════╗                │
          ┌────────║  VACANT    ║ ◄──────────────┤
          │        ║  (Trống)   ║                │
          │        ╚═════╤═════╝                │
          │              │                       │
          │    check_in()│                       │
          │              ▼                       │
          │        ╔═══════════╗                │
          │        ║ OCCUPIED  ║                │
          │        ║(Có khách) ║                │
          │        ╚═════╤═════╝                │
          │              │                       │
          │   check_out()│                       │
          │              ▼                       │
          │        ╔═══════════╗                │
          │        ║ CLEANING  ║                │
          │        ║(Cần dọn)  ║                │
          │        ╚═════╤═════╝                │
          │              │                       │
          │  update_     │                       │
          │  housekeeping│("clean")              │
          │              └───────────────────────┘
          │
          │    (Reservations - v2)
          │        ╔═══════════╗
          └───────►║  BOOKED   ║ ──── check_in() ──► OCCUPIED
                   ║(Đặt trước)║
                   ╚═══════════╝
```

**Transitions chi tiết:**

| From | To | Trigger | Command |
|------|----|---------|---------|
| `vacant` | `occupied` | Khách check-in | `check_in()` |
| `occupied` | `cleaning` | Khách check-out | `check_out()` |
| `cleaning` | `vacant` | Dọn phòng xong | `update_housekeeping(status="clean")` |
| `vacant` | `booked` | Đặt phòng trước | *(v2 - chưa implement)* |
| `booked` | `occupied` | Khách đến check-in | `check_in()` |

---

### 3.2 Booking Status State Machine

```
                    ╔═══════════════╗
      check_in() ──►║    ACTIVE      ║
                    ║  (Đang ở)      ║
                    ╚════════╤══════╝
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     extend_stay()    check_out()     (cancel - v2)
              │              │              │
              │              ▼              ▼
              │    ╔═══════════════╗  ╔════════════╗
              └───►║  CHECKED_OUT  ║  ║ CANCELLED  ║
                   ║  (Đã trả)     ║  ║ (Đã hủy)  ║
                   ╚═══════════════╝  ╚════════════╝
```

**Ghi chú:** `extend_stay()` giữ nguyên status `active`, chỉ update `nights`, `expected_checkout`, và `total_price`.

---

### 3.3 Housekeeping Task State Machine

```
   ╔════════════════════╗
   ║  NEEDS_CLEANING    ║ ◄── Auto-created khi check_out()
   ║  🟡 (Cần dọn)      ║
   ╚═════════╤══════════╝
             │
             │ update_housekeeping("cleaning")
             ▼
   ╔════════════════════╗
   ║  CLEANING          ║
   ║  🔄 (Đang dọn)     ║
   ╚═════════╤══════════╝
             │
             │ update_housekeeping("clean")
             ▼
   ╔════════════════════╗
   ║  CLEAN             ║ ──► Room status → VACANT
   ║  🟢 (Sạch)         ║
   ╚════════════════════╝
```

---

### 3.4 Authentication State Machine

```
                    ╔═══════════════╗
    App start ─────►║  LOGGED_OUT   ║ ◄─── logout()
                    ╚════════╤══════╝
                             │
                     login(pin) ──► verify PIN hash
                             │
                    ┌────────┴────────┐
                    │                 │
               ✅ Success          ❌ Fail
                    │                 │
                    ▼                 ▼
           ╔═══════════════╗    Error message
           ║  LOGGED_IN    ║    "PIN không đúng"
           ║  (role-based) ║
           ╚═══════════════╝
                    │
            ┌───────┴───────┐
            │               │
        admin          receptionist
            │               │
       Full access     Limited access
     (Settings, CRUD)  (Check-in/out only)
```

---

### 3.5 OCR Pipeline State Machine

```
   ╔═══════════════════╗
   ║  IDLE / WATCHING  ║ ◄── watcher.rs (luôn chạy background)
   ║  ~/MHM/Scans/     ║
   ╚═════════╤═════════╝
             │
             │ notify::EventKind::Create
             │ (File mới: .jpg/.png/.tiff)
             ▼
   ╔═══════════════════╗
   ║  DEBOUNCE         ║ ── sleep 500ms (đợi file write xong)
   ╚═════════╤═════════╝
             │
             ▼
   ╔═══════════════════╗
   ║  OCR PROCESSING   ║ ── ocr::ocr_image(engine, path)
   ║  (PaddleOCR v5)   ║ ── ~200-300ms trên Apple Silicon
   ╚═════════╤═════════╝
             │
        ┌────┴────┐
        │         │
     ✅ OK     ❌ Fail
        │         │
        ▼         ▼
   parse_cccd()   emit("ocr-error")
        │
        ▼
   emit("ocr-result", CccdInfo)
        │
        ▼  (Frontend nhận event)
   ╔═══════════════════╗
   ║  SHOW POPUP       ║ ── Hiện thông tin khách
   ║  (CheckinSheet)   ║ ── Cho chọn phòng + confirm
   ╚═══════════════════╝
```

---

## 4. Data Pipeline Diagrams

### 4.1 Check-in Data Pipeline (Complete Flow)

```
 ┌───────────┐    ┌─────────────┐    ┌──────────────┐    ┌──────────────┐
 │ CCCD Scan  │    │ File Watcher │    │  OCR Engine  │    │  CCCD Parser │
 │ (Canon)    │───►│  (notify)    │───►│ (PaddleOCR)  │───►│  (regex)     │
 └───────────┘    └─────────────┘    └──────────────┘    └──────┬───────┘
                                                                │
                  ┌──── Tauri Event ◄──── emit("ocr-result") ──┘
                  │
                  ▼
 ┌────────────────────────────────────────────────────────────────────┐
 │  Frontend: CheckinSheet.tsx                                        │
 │  ┌──────────┐  ┌──────────────┐  ┌────────────┐  ┌──────────────┐│
 │  │ OCR Data │  │ Manual Entry │  │ Room Select │  │ Guest Count  ││
 │  │ Auto-fill│  │ Fix/Override │  │ (Dropdown)  │  │ Nights       ││
 │  └────┬─────┘  └──────┬───────┘  └──────┬─────┘  └──────┬───────┘│
 │       └───────────┬────┘                │               │        │
 │                   ▼                     ▼               ▼        │
 │           CheckInRequest { room_id, guests[], nights, source }   │
 └────────────────────────────────┬───────────────────────────────────┘
                                  │
                          invoke("check_in", req)
                                  │
                                  ▼
 ┌────────────────────────────────────────────────────────────────────┐
 │  Backend: commands::check_in()                                     │
 │                                                                    │
 │  1. Validate room.status == "vacant"                               │
 │  2. Load PricingRule for room_type                                │
 │  3. calculate_price_preview() → PricingResult                      │
 │  4. ┌─── FOR each guest in req.guests ───┐                       │
 │     │  INSERT INTO guests (uuid, name...) │                        │
 │     └────────────────────────────────────┘                        │
 │  5. INSERT INTO bookings (uuid, room_id, guest_id, total_price)   │
 │  6. INSERT INTO booking_guests (booking_id, guest_id) × N         │
 │  7. UPDATE rooms SET status = "occupied" WHERE id = room_id       │
 │  8. INSERT INTO transactions (booking_id, paid_amount)             │
 │  9. emit("db-updated", "rooms")                                    │
 │                                                                    │
 │  RETURN → Booking                                                  │
 └────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
 ┌────────────────────────────────────────────────────────────────────┐
 │  Frontend: useHotelStore                                           │
 │  ├── fetchRooms()    → GET rooms (refresh dashboard colors)       │
 │  ├── fetchStats()    → GET dashboard stats (occupied count, $$$)  │
 │  └── setTab("dashboard") → navigate back                          │
 └────────────────────────────────────────────────────────────────────┘
```

---

### 4.2 Check-out Data Pipeline

```
 ┌──────────────┐         ┌───────────────────────────────────────────┐
 │ RoomDetail   │         │  commands::check_out()                    │
 │ Click        │──req───►│                                           │
 │ "Check-out"  │         │  1. Verify booking.status == "active"     │
 └──────────────┘         │  2. UPDATE bookings SET                   │
                          │     status = "checked_out"                │
                          │     actual_checkout = NOW()               │
                          │  3. If final_paid > 0:                    │
                          │     INSERT INTO transactions              │
                          │  4. UPDATE rooms SET status = "cleaning"  │
                          │  5. INSERT INTO housekeeping              │
                          │     (status = "needs_cleaning")           │
                          │  6. emit("db-updated", "rooms")           │
                          └───────────────────────────────────────────┘
                                         │
                                         ▼
                          ┌──────────────────────────────┐
                          │ Auto-cascade effects:         │
                          │ • Dashboard: phòng → 🟡       │
                          │ • Housekeeping: task mới      │
                          │ • Stats: update revenue       │
                          └──────────────────────────────┘
```

---

### 4.3 Pricing Engine Pipeline

```
 ┌────────────────────────────┐
 │  Input                      │
 │  ├── room_type ("deluxe")   │
 │  ├── check_in  (datetime)   │
 │  ├── check_out (datetime)   │
 │  └── pricing_type           │
 │      ("hourly"|"overnight"  │
 │       |"daily"|"nightly")   │
 └──────────────┬──────────────┘
                │
                ▼
 ┌────────────────────────────────────────────────────┐
 │  1. Load PricingRule from DB (by room_type)         │
 │     ├── hourly_rate      (₫/hour)                  │
 │     ├── overnight_rate   (₫/night)                 │
 │     ├── daily_rate       (₫/day)                   │
 │     ├── overnight_start  ("22:00")                 │
 │     ├── overnight_end    ("11:00")                 │
 │     ├── early_checkin_pct (30%)                    │
 │     ├── late_checkout_pct (30%)                    │
 │     └── weekend_uplift_pct (20%)                   │
 └──────────────┬─────────────────────────────────────┘
                │
                ▼
 ┌────────────────────────────────────────────────────┐
 │  2. Route to pricing algorithm:                     │
 │                                                     │
 │  "hourly"    → calculate_hourly()                  │
 │               ├── hours × hourly_rate               │
 │               └── AUTO-CAP: if > overnight → cap   │
 │                                                     │
 │  "overnight" → calculate_overnight()               │
 │               ├── nights × overnight_rate           │
 │               ├── + early_checkin surcharge?         │
 │               └── + late_checkout surcharge?         │
 │                                                     │
 │  "daily"     → calculate_daily()                   │
 │               ├── days × daily_rate                 │
 │               ├── + early/late surcharges           │
 │               └── + weekend uplift                   │
 │                                                     │
 │  "nightly"   → calculate_nightly() (legacy)        │
 │               └── nights × base_price               │
 └──────────────┬─────────────────────────────────────┘
                │
                ▼
 ┌────────────────────────────────────────────────────┐
 │  3. Apply modifiers:                                │
 │     ├── Weekend uplift (Fri/Sat/Sun +20%)          │
 │     ├── Special dates uplift (Tết, holidays +X%)   │
 │     └── Extra person fee (if guests > max_guests)   │
 └──────────────┬─────────────────────────────────────┘
                │
                ▼
 ┌────────────────────────────────────────────────────┐
 │  Output: PricingResult                              │
 │  ├── total:  380,000₫                              │
 │  ├── lines:  [ "1 night × 350,000₫",              │
 │  │             "Late checkout +30,000₫" ]          │
 │  └── label:  "380.000₫"                           │
 └────────────────────────────────────────────────────┘
```

---

### 4.4 Night Audit Pipeline

```
 ┌──────────────┐
 │ Trigger:      │
 │ "Run Audit"   │──── invoke("run_night_audit", { audit_date, notes })
 │ (NightAudit   │
 │  page)        │
 └──────┬───────┘
        │
        ▼
 ┌─────────────────────────────────────────────────────────┐
 │  commands::run_night_audit()                             │
 │                                                          │
 │  1. Count rooms by status (occupied/vacant/cleaning)     │
 │  2. SUM bookings revenue for audit_date                  │
 │  3. SUM transactions for audit_date                      │
 │  4. SUM expenses for audit_date                          │
 │  5. List active bookings checking out today              │
 │  6. INSERT INTO audit_logs (JSON summary)                │
 │                                                          │
 │  Output: {                                               │
 │    rooms_occupied, rooms_vacant,                         │
 │    revenue_total, expenses_total,                        │
 │    net_revenue, bookings_checked_out,                    │
 │    bookings_still_active, discrepancies[]                │
 │  }                                                       │
 └─────────────────────────────────────────────────────────┘
```

---

### 4.5 Realtime Event Pipeline

```
 ┌─────────────────────────────────────────────────────────────────┐
 │  Backend Events (Tauri Emitter)                                  │
 │                                                                  │
 │  check_in()           ──► emit("db-updated", "rooms")           │
 │  check_out()          ──► emit("db-updated", "rooms")           │
 │  update_housekeeping()──► emit("db-updated", "housekeeping")    │
 │  update_room()        ──► emit("db-updated", "rooms")           │
 │  create_room()        ──► emit("db-updated", "rooms")           │
 │  delete_room()        ──► emit("db-updated", "rooms")           │
 │  save_pricing_rule()  ──► emit("db-updated", "pricing")         │
 │  add_folio_line()     ──► emit("db-updated", "folio")           │
 │  run_night_audit()    ──► emit("db-updated", "audit")           │
 │  watcher (OCR done)   ──► emit("ocr-result", CccdInfo)         │
 │  watcher (OCR fail)   ──► emit("ocr-error", msg)                │
 └───────────────────────────────┬─────────────────────────────────┘
                                 │
                                 ▼
 ┌─────────────────────────────────────────────────────────────────┐
 │  Frontend: App.tsx — listen("db-updated")                        │
 │                                                                  │
 │  useEffect(() => {                                               │
 │    listen("db-updated", () => {                                  │
 │      fetchRooms()                                                │
 │      fetchStats()                                                │
 │    })                                                            │
 │  })                                                              │
 │                                                                  │
 │  → Dashboard auto-refresh khi bất kỳ data thay đổi              │
 └─────────────────────────────────────────────────────────────────┘
```

---

## 5. Entity Relationship Diagram (ERD)

```
┌─────────────────┐       ┌─────────────────────┐
│   room_types    │       │       rooms          │
├─────────────────┤       ├─────────────────────┤
│ id (PK)         │◄──────│ type (FK → name)     │
│ name            │       │ id (PK) "1A","2B"    │
│ created_at      │       │ name, floor          │
└─────────────────┘       │ has_balcony          │
                          │ base_price           │
                          │ max_guests           │
                          │ extra_person_fee     │
                          │ status               │
                          └──────────┬──────────┘
                                     │ 1
                                     │
                                     │ N
                          ┌──────────▼──────────┐         ┌───────────────────┐
                          │     bookings         │         │     guests        │
                          ├─────────────────────┤         ├───────────────────┤
                          │ id (PK)              │    ┌───►│ id (PK)           │
                          │ room_id (FK)─────────┘    │    │ full_name         │
                          │ primary_guest_id (FK)─────┘    │ doc_number (CCCD) │
                          │ check_in_at          │         │ guest_type        │
                          │ expected_checkout     │         │ dob, gender       │
                          │ actual_checkout       │         │ nationality       │
                          │ nights               │         │ address           │
                          │ total_price          │         │ phone             │
                          │ paid_amount          │         │ scan_path         │
                          │ status               │         └───────┬───────────┘
                          │ source               │                 │
                          └──────┬───────────────┘                 │
                                 │ 1                               │
                     ┌───────────┼───────────────┐                 │
                     │           │               │                 │
                     │ N         │ N             │ N               │
          ┌──────────▼───┐  ┌───▼──────────┐  ┌─▼────────────────▼─┐
          │ transactions │  │ folio_lines  │  │  booking_guests     │
          ├──────────────┤  ├──────────────┤  ├─────────────────────┤
          │ id (PK)      │  │ id (PK)      │  │ booking_id (FK,PK) │
          │ booking_id   │  │ booking_id   │  │ guest_id (FK,PK)   │
          │ amount       │  │ category     │  └─────────────────────┘
          │ type         │  │ description  │
          │ note         │  │ amount       │
          │ created_at   │  └──────────────┘
          └──────────────┘

                    ┌────────────────┐         ┌─────────────────────┐
                    │   expenses     │         │   housekeeping      │
                    ├────────────────┤         ├─────────────────────┤
                    │ id (PK)        │         │ id (PK)             │
                    │ category       │         │ room_id (FK→rooms)  │
                    │ amount         │         │ status              │
                    │ note           │         │ note                │
                    │ expense_date   │         │ triggered_at        │
                    │ created_at     │         │ cleaned_at          │
                    └────────────────┘         └─────────────────────┘

 ┌──────────────────┐  ┌────────────────────┐  ┌─────────────────────┐
 │   users          │  │  pricing_rules     │  │   special_dates     │
 ├──────────────────┤  ├────────────────────┤  ├─────────────────────┤
 │ id (PK)          │  │ room_type (PK)     │  │ date (PK)           │
 │ name             │  │ hourly_rate        │  │ label               │
 │ pin_hash         │  │ overnight_rate     │  │ uplift_pct          │
 │ role             │  │ daily_rate         │  └─────────────────────┘
 │ active           │  │ overnight_start    │
 └──────────────────┘  │ overnight_end      │  ┌─────────────────────┐
                       │ early_pct          │  │   audit_logs        │
                       │ late_pct           │  ├─────────────────────┤
                       │ weekend_pct        │  │ id (PK)             │
                       └────────────────────┘  │ audit_date          │
                                               │ data (JSON)         │
                                               │ notes               │
                    ┌────────────────────┐     └─────────────────────┘
                    │   settings (KV)    │
                    ├────────────────────┤
                    │ key (PK)           │
                    │ value (JSON text)  │
                    └────────────────────┘
```

---

## 6. Frontend Page Map & Navigation

```
┌─────────────────────────────────────────────────────────┐
│                      App.tsx                             │
│                  (Sidebar Layout)                        │
├──────────────┬──────────────────────────────────────────┤
│   SIDEBAR    │              PAGE CONTENT                 │
│              │                                           │
│  ┌─────────┐ │  ┌───────────────────────────────────┐   │
│  │Dashboard│ │  │ Dashboard.tsx                      │   │
│  │         │─┼─►│ ├── 10 RoomCards (grid)            │   │
│  │         │ │  │ ├── DashboardStats (top bar)       │   │
│  │         │ │  │ └── Recent Activity                │   │
│  ├─────────┤ │  └───────────────────────────────────┘   │
│  │Reserve. │ │                                           │
│  │         │─┼─► Reservations.tsx — Booking list table  │
│  ├─────────┤ │                                           │
│  │Rooms    │─┼─► Rooms.tsx — Room config CRUD            │
│  ├─────────┤ │                                           │
│  │Guests   │─┼─► Guests.tsx — Guest search + history    │
│  ├─────────┤ │                                           │
│  │Housekp. │─┼─► Housekeeping.tsx — Task list            │
│  ├─────────┤ │                                           │
│  │Analytics│─┼─► Analytics.tsx — Charts + reports        │
│  ├─────────┤ │                                           │
│  │Audit    │─┼─► NightAudit.tsx — End-of-day             │
│  ├─────────┤ │                                           │
│  │Settings │─┼─► Settings.tsx — Hotel info, pricing..   │
│  └─────────┘ │                                           │
│              │  ┌───────────────────────────────────┐   │
│              │  │ RoomDetail.tsx (overlay/sub-page)  │   │
│              │  │ ├── Guest info, booking details    │   │
│              │  │ ├── Check-out, Extend stay         │   │
│              │  │ └── Copy lưu trú                   │   │
│              │  └───────────────────────────────────┘   │
│              │                                           │
│              │  ┌───────────────────────────────────┐   │
│              │  │ CheckinSheet.tsx (slide-over)      │   │
│              │  │ ├── OCR auto-fill guest info       │   │
│              │  │ ├── Room select + nights           │   │
│              │  │ └── Confirm check-in               │   │
│              │  └───────────────────────────────────┘   │
├──────────────┴──────────────────────────────────────────┤
│  LoginScreen.tsx (shown when !isAuthenticated)           │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Tauri IPC Command Registry (40+ commands)

### Core Operations
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `get_rooms` | — | `Room[]` | Lấy danh sách phòng |
| `get_dashboard_stats` | — | `DashboardStats` | Stats cho dashboard |
| `check_in` | `CheckInRequest` | `Booking` | Check-in khách |
| `check_out` | `CheckOutRequest` | `()` | Check-out + auto housekeeping |
| `extend_stay` | `booking_id` | `Booking` | Thêm 1 đêm |
| `get_room_detail` | `room_id` | `RoomWithBooking` | Chi tiết phòng + booking + guests |

### Guest Management
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `get_all_guests` | `search?` | `GuestSummary[]` | Tìm khách |
| `get_guest_history` | `guest_id` | `GuestHistoryResponse` | Lịch sử lưu trú |
| `search_guest_by_phone` | `phone` | `GuestSummary[]` | Quick lookup |

### Room Config
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `create_room` | `CreateRoomRequest` | `Room` | Thêm phòng |
| `update_room` | `UpdateRoomRequest` | `Room` | Sửa phòng |
| `delete_room` | `room_id` | `()` | Xóa phòng |
| `get_room_types` | — | `RoomType[]` | Loại phòng |
| `create_room_type` | `CreateRoomTypeRequest` | `RoomType` | Thêm loại |
| `delete_room_type` | `room_type_id` | `()` | Xóa loại |

### Analytics & Reports
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `get_analytics` | `period` | `AnalyticsData` | Dashboard analytics |
| `get_revenue_stats` | `from, to` | `RevenueStats` | Revenue by period |
| `get_recent_activity` | `limit` | `ActivityItem[]` | Recent actions |
| `get_all_bookings` | `BookingFilter?` | `BookingWithGuest[]` | All reservations |

### Pricing
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `get_pricing_rules` | — | `Value[]` | Bảng giá |
| `save_pricing_rule` | 12 params | `()` | Lưu rule pricing |
| `calculate_price_preview` | room_type, dates | `PricingResult` | Preview giá |
| `get_special_dates` | — | `Value[]` | Ngày lễ |
| `save_special_date` | date, label, pct | `()` | Thêm ngày lễ |

### Folio & Billing
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `add_folio_line` | booking, category, amount | `Value` | Thêm dòng phí |
| `get_folio_lines` | `booking_id` | `Value[]` | Lấy folio |

### Night Audit
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `run_night_audit` | date, notes | `Value` | Chạy audit cuối ngày |
| `get_audit_logs` | — | `Value[]` | Lịch sử audit |

### Settings, Auth, Export
| Command | Input | Output | Mô tả |
|---------|-------|--------|-------|
| `login` | `LoginRequest` | `LoginResponse` | Đăng nhập |
| `logout` | — | `()` | Đăng xuất |
| `get_current_user` | — | `User?` | Session check |
| `list_users` | — | `User[]` | DS users |
| `create_user` | `CreateUserRequest` | `User` | Tạo user |
| `save_settings` | key, value | `()` | Lưu setting |
| `get_settings` | key | `String?` | Đọc setting |
| `backup_database` | — | `String` | Backup → `~/MHM/backups/` |
| `export_csv` | — | `String` | Export all data |
| `export_bookings_csv` | from?, to? | `String` | Export bookings |
| `scan_image` | path | `CccdInfo` | Manual OCR scan |

---

## 8. Database Migration History

```
v1  → Core tables (rooms, guests, bookings, transactions, expenses, housekeeping)
v2  → settings (KV store)
v3  → users + pin_hash + role
v4  → guests.phone column
v5  → pricing_rules table
v6  → special_dates table
v7  → folio_lines table
v8  → audit_logs table
v9  → room_types table + rooms.max_guests + rooms.extra_person_fee
v10 → booking_guests → CASCADE delete
```

---

*Tài liệu này map toàn bộ kiến trúc MHM từ frontend đến database. Sử dụng cho onboarding, debugging, và planning.*
