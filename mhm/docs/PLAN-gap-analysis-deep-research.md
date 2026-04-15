# GAP ANALYSIS & ROADMAP: Deep Research vs MHM — Final V2.0

> **Ngày tạo:** 15/03/2026 | **Cập nhật:** 15/03/2026  
> **Nguồn:** `Deep-Research-Hotel-Manager.md` ↔ Codebase MHM (Tauri + React + Rust)  
> **Trạng thái tổng:** ~35% MVP hoàn thành  
> **Version:** V2.0 — Đã tích hợp feedback từ DeepThink Audit

---

## HIỆN TRẠNG MHM

| Layer | Stack |
|-------|-------|
| Frontend | React + TypeScript + Vite + shadcn/ui + Recharts |
| Backend | Rust + Tauri + SQLite (sqlx) |
| OCR | Custom Rust OCR engine (`ocr.rs`) |
| i18n | Custom vi/en (`lib/i18n.ts`) |
| Pages | Dashboard, Rooms, RoomDetail, Reservations, Guests, Housekeeping, Analytics, Statistics, Settings, Timeline |
| DB Tables | rooms, guests, bookings, booking_guests, transactions, expenses, housekeeping, settings |
| Rust Commands | 44 commands (check-in/out, rooms, guests, housekeeping, expenses, analytics, OCR, settings, CSV export) |

---

## I. TIER 1 — MUST-HAVE: ĐỐI CHIẾU CHI TIẾT

| # | Tính năng | Status | % | GAP & Giải pháp |
|---|-----------|--------|---|-----------------|
| 1 | **Sơ đồ phòng real-time** | ⚠️ | 80% | Có grid + mã màu 4 trạng thái. **GAP:** Cần **Tauri Events** (event-driven) thay vì polling — Rust emit event khi DB thay đổi, React auto re-fetch. |
| 2 | **Pricing Engine VN** | ❌ | 5% | Hiện chỉ `base_price × nights`. **GAP LỚN NHẤT:** Cần rule engine: giá giờ/đêm/ngày, capping tự động, surcharge sớm/trễ, giá cuối tuần/lễ. **Bắt buộc Price Snapshot** — snapshot pricing rules vào booking (JSON) khi check-in để tránh nhảy giá. |
| 3 | **Check-in/out flow** | ⚠️ | 70% | Có đầy đủ nhưng form quá dài. **GAP:** Cần mode "Quick Walk-in" (Tên + SĐT + Số đêm, < 15 giây). OCR bổ sung thông tin sau. |
| 4 | **Folio / Billing** | ❌ | 20% | Chỉ có tiền phòng cơ bản. **GAP:** Thiếu table `services`/`folio_items` cho dịch vụ phát sinh (minibar, giặt ủi). Thiếu `payment_method` (tiền mặt/chuyển khoản/QR). Cần Folio UI gộp tất cả. Tích hợp **VietQR động** (quick win mạnh). |
| 5 | **In bill nhiệt** | ❌ | 0% | Chưa có. **Defer** — Anh sẽ tự implement ESC/POS (Epson) sau. Không block MVP. |
| 6 | **Phân quyền RBAC** | ❌ | 0% | Không có login, ai cũng full quyền. **GAP:** Cần table `users`, login PIN 4 số, RBAC middleware trong Rust commands, `created_by` trong mọi bảng giao dịch. |
| 7 | **Báo cáo doanh thu** | ✅ | 85% | Analytics tốt (daily revenue, occupancy, ADR, RevPAR, top rooms, expenses). **GAP nhỏ:** Chưa tách phòng vs dịch vụ. |
| 8 | **Night Audit** | ❌ | 0% | Hoàn toàn chưa có. **GAP:** Cần flow chốt ca, đếm tiền mặt, **Data Lock** (khóa sổ ngày đã chốt — read-only). |
| 9 | **Guest Profile** | ⚠️ | 75% | Có danh sách, search, VIP badge, lịch sử. **GAP:** DB thiếu `phone` và `notes` — rất quan trọng. Cần auto-suggest khách cũ khi gõ SĐT. |
| 10 | **Backup/Restore** | ❌ | 15% | Chỉ có CSV export ≠ backup. **GAP:** Cần Rust background task chạy `VACUUM INTO 'backup.db'` định kỳ. Restore = replace DB file. |

---

## II. TIER 2 — NICE-TO-HAVE

| # | Tính năng | Status | Ghi chú |
|---|-----------|--------|---------|
| 1 | Gantt Reservation Calendar | ⚠️ | Có timeline view, chưa drag-drop |
| 2 | OCR đọc CCCD | ✅ | Custom Rust OCR engine hoạt động |
| 3 | Khai báo tạm trú | ❌ | Có copy text, chưa export Excel chuẩn CA |
| 4 | Channel Manager (OTA) | ❌ | Post-MVP |
| 5 | E-invoice | ❌ | Post-MVP |
| 6 | IoT điện phòng | ❌ | Không làm |
| 7 | AI Assistant | ❌ | Không ưu tiên |
| 8 | NFC chip CCCD | ❌ | Cần hardware chuyên dụng |
| 9 | Auto-update | ❌ | Cần cấu hình Tauri updater |
| 10 | Dark Mode | ✅ | Đã có |
| 11 | Multi-language vi/en | ✅ | Đã có |
| 12 | Toast system | ✅ | Sonner đã tích hợp |

---

## III. BLIND SPOTS

| # | Vấn đề | Status | Priority |
|---|--------|--------|----------|
| 1 | Night Audit | ❌ | 🔴 Critical — Phase 4 |
| 2 | Guest phone + notes | ⚠️ | 🔴 Phase 1 |
| 3 | True Backup/Restore | ❌ | 🟠 Phase 5 |
| 4 | Multi-device conflict | ❌ | 🟡 Post-MVP (WAL đã bật) |
| 5 | Data migration/import | ❌ | 🟡 Phase 5 |
| 6 | Group Booking | ❌ | 🟡 Post-MVP |

---

## IV. KIẾN TRÚC QUYẾT ĐỊNH (Đã thống nhất)

| Quyết định | Approach | Lý do |
|------------|----------|-------|
| **Real-time update** | Tauri Events (event-driven) | Polling là anti-pattern cho desktop, hao CPU + DB lock risk |
| **RBAC timing** | Phase 1 (Foundation) | Tránh retrofit `created_by` vào 44 commands sau này |
| **Price Snapshot** | JSON snapshot vào bookings | Tránh nhảy giá khi admin đổi bảng giá |
| **DB Migrations** | Versioned inline migrations + `schema_version` table | `sqlx migrate` CLI quá nặng cho desktop app distribution |
| **In bill (MVP)** | Defer — anh tự implement ESC/POS Epson | HTML print fallback nếu cần nhanh, ESC/POS cho production |
| **Backup** | `VACUUM INTO` (Rust background task) | Copy an toàn 100% SQLite database |

---

## V. ROADMAP IMPLEMENTATION — 15 TUẦN

### 🧱 Phase 1: Foundation + RBAC (Tuần 1-3)

> Xây móng vững, tránh nợ kỹ thuật.

**DB Migrations (versioned inline):**
- [ ] Bảng `schema_version` cho versioned migrations
- [ ] Bảng `users` (id, name, pin_hash, role: admin/receptionist, active, created_at)
- [ ] Bảng `audit_logs` (id, user_id, action, entity_type, entity_id, old_value, new_value, created_at)
- [ ] `guests` + `phone TEXT`, `notes TEXT`
- [ ] `transactions` + `payment_method TEXT`, `created_by TEXT`
- [ ] `bookings` + `created_by TEXT`

**Auth & RBAC:**
- [ ] Login screen — PIN 4 số
- [ ] User session management (Zustand store)
- [ ] Rust middleware: inject `user_id` vào mọi write command
- [ ] Permission checks: receptionist không sửa giá, xóa giao dịch

**Tauri Events:**
- [ ] Rust side: `app_handle.emit_all("db_updated", payload)` sau mỗi write command
- [ ] React side: `listen("db_updated")` → auto re-fetch relevant data
- [ ] Áp dụng cho Dashboard, Rooms, Housekeeping

**Quick Check-in:**
- [ ] Mode rút gọn: Tên + SĐT + Số đêm + Loại phòng
- [ ] OCR nhúng vào flow bổ sung thông tin (không bắt buộc)
- [ ] Auto-suggest khách cũ khi gõ SĐT

---

### 🔥 Phase 2: VN Pricing Engine (Tuần 4-7)

> Trái tim hệ thống. Pure Rust, KHÔNG tính tiền trên UI.

**Schema:**
- [ ] Bảng `pricing_rules` (room_type, pricing_type, hourly_rate, overnight_rate, daily_rate, configs JSON)
- [ ] Bảng `surcharge_rules` (early_checkin tiers, late_checkout tiers)
- [ ] Bảng `special_dates` (weekend override, holiday pricing)

**Rust Logic (`pricing.rs`):**
- [ ] Pricing types: hourly, overnight, daily
- [ ] Capping tự động: hourly tích lũy > overnight → chuyển overnight
- [ ] Surcharge: early check-in (5-9h: +50%, 9-14h: +30%), late check-out (12-15h: +30%, 15-18h: +50%, >18h: +100%)
- [ ] Weekend/holiday rate override
- [ ] **Price Snapshot**: lưu copy cứng rules vào `bookings.pricing_snapshot` (JSON) khi check-in

**TDD (bắt buộc):**
- [ ] ≥15 unit tests trong Rust:
  - Nghỉ theo giờ (1h, 2h, 3h, 5h)
  - Qua đêm (22h → 10h sáng)
  - Theo ngày (14h → 12h hôm sau)
  - Capping: giá giờ vượt giá đêm → tự chuyển
  - Phụ thu sớm/trễ
  - Đổi phòng giữa chừng
  - Giá cuối tuần / lễ Tết

**Settings UI:**
- [ ] Trang cấu hình pricing rules per room type
- [ ] Preview tính giá test (nhập giờ vào/ra → xem giá)

---

### 💰 Phase 3: Folio, Billing & Payment (Tuần 8-10)

**Schema:**
- [ ] Bảng `services` (id, booking_id, name, quantity, unit_price, total, created_by, created_at)
- [ ] Hoặc `folio_items` gom chung room charges + services

**Folio UI:**
- [ ] Trang tổng hợp khi Check-out: Tiền phòng (từ pricing engine) + Dịch vụ phát sinh − Đã cọc = Tiền cần thu
- [ ] Thêm dịch vụ phát sinh inline (minibar, giặt ủi, etc.)
- [ ] Nút chọn payment method: Tiền mặt / Chuyển khoản

**VietQR (Quick Win):**
- [ ] Sinh VietQR động chứa sẵn số tiền khi chọn Chuyển khoản
- [ ] Cấu hình bank account trong Settings

---

### 🔒 Phase 4: Night Audit & Data Lock (Tuần 11-13)

**Night Audit Page:**
- [ ] Màn hình bàn giao ca
- [ ] Tổng phòng bán, tổng tiền thu, phân tách tiền mặt/chuyển khoản
- [ ] Đếm tiền mặt trong két → so khớp với system
- [ ] Log lịch sử chốt ca (ai chốt, lúc nào, chênh lệch bao nhiêu)

**Data Lock (Đóng sổ):**
- [ ] Bảng `day_closings` (date, closed_by, closed_at, cash_counted, system_total, variance)
- [ ] Khóa read-only toàn bộ giao dịch thuộc ngày đã đóng sổ
- [ ] Middleware Rust: reject mọi edit/delete cho ngày đã close

**Refactor Analytics:**
- [ ] Dashboard tách riêng Doanh thu phòng vs Doanh thu dịch vụ
- [ ] Biểu đồ payment method breakdown

---

### 🛡️ Phase 5: Backup, Import & Polish (Tuần 14-15)

**True Backup/Restore:**
- [ ] Rust background task (`tokio::time::interval`) chạy `VACUUM INTO` hàng ngày
- [ ] Backup location configurable trong Settings
- [ ] Nút Restore 1-click (replace DB file + restart)

**Khai báo tạm trú:**
- [ ] Export Excel đúng format chuẩn Công An VN
- [ ] Auto-fill từ guest data

**Data Migration:**
- [ ] Import danh sách phòng từ Excel
- [ ] Import danh sách khách hàng cũ từ Excel

**Other:**
- [ ] Auto-update: cấu hình Tauri updater
- [ ] Group Booking cơ bản (nếu kịp)

---

## VI. RISK ASSESSMENT

| Risk | Level | Mitigation |
|------|-------|------------|
| Pricing Engine bugs | 🔴 Cao | TDD bắt buộc, ≥15 unit tests pass trước khi merge. Logic tách biệt trong `pricing.rs`, không dính UI |
| Mất dữ liệu DB local | 🔴 Cao | `VACUUM INTO` backup hàng ngày. Recommend user backup ra ổ khác / Google Drive |
| RBAC retrofit 44 commands | 🟠 TB | Làm Phase 1 nên chỉ cần làm 1 lần đúng, middleware pattern |
| SQLite concurrency | 🟡 Thấp | MVP single-device. WAL mode đã bật (`PRAGMA journal_mode=WAL`) |
| Thermal print compatibility | 🟡 Thấp | Defer — anh tự implement ESC/POS Epson sau |

---

## VII. KẾT LUẬN

> **MHM ~35% MVP.** Kiến trúc Tauri + Rust + SQLite rất vững — nhẹ, không cần server, không tốn phí duy trì.
> 
> **Ưu tiên tuyệt đối:** Phase 1 (RBAC Foundation) → Phase 2 (Pricing Engine VN).
> 
> Pricing Engine tính tiền chính xác nhất cho thị trường VN = selling point duy nhất thắng mọi PMS quốc tế.