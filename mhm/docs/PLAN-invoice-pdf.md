# Invoice PDF System — Implementation Plan

Tạo hệ thống invoice PDF chuyên nghiệp cho Hotel Manager, sử dụng `@react-pdf/renderer` ở frontend và Rust data layer ở backend. Invoice dạng "Confirmation" — gửi cho khách khi đặt phòng hoặc check-in.

## Proposed Changes

### Backend — Rust Data Layer

---

#### [MODIFY] [db.rs](mhm/src-tauri/src/db.rs)

Thêm **V8 migration** — bảng `invoices`:

```sql
CREATE TABLE IF NOT EXISTS invoices (
    id              TEXT PRIMARY KEY,
    invoice_number  TEXT NOT NULL UNIQUE,
    booking_id      TEXT NOT NULL REFERENCES bookings(id),
    hotel_name      TEXT NOT NULL,
    hotel_address   TEXT NOT NULL,
    hotel_phone     TEXT NOT NULL,
    guest_name      TEXT NOT NULL,
    guest_phone     TEXT,
    room_name       TEXT NOT NULL,
    room_type       TEXT NOT NULL,
    check_in        TEXT NOT NULL,
    check_out       TEXT NOT NULL,
    nights          INTEGER NOT NULL,
    pricing_breakdown TEXT NOT NULL,  -- JSON: PricingLine[]
    subtotal        REAL NOT NULL,
    deposit_amount  REAL NOT NULL DEFAULT 0,
    total           REAL NOT NULL,
    balance_due     REAL NOT NULL,
    policy_text     TEXT,
    notes           TEXT,
    status          TEXT NOT NULL DEFAULT 'issued',
    created_at      TEXT NOT NULL
);
```

> **Invoice number format**: `INV-YYYYMMDD-XXX` (auto-increment per ngày, VD: `INV-20260317-001`)

---

#### [MODIFY] [models.rs](mhm/src-tauri/src/models.rs)

Thêm struct:

```rust
pub struct InvoiceData {
    pub id: String,
    pub invoice_number: String,
    pub booking_id: String,
    pub hotel_name: String,
    pub hotel_address: String,
    pub hotel_phone: String,
    pub guest_name: String,
    pub guest_phone: Option<String>,
    pub room_name: String,
    pub room_type: String,
    pub check_in: String,
    pub check_out: String,
    pub nights: i32,
    pub pricing_breakdown: Vec<PricingLine>,  // reuse from pricing.rs
    pub subtotal: f64,
    pub deposit_amount: f64,
    pub total: f64,
    pub balance_due: f64,
    pub policy_text: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
}
```

---

#### [MODIFY] [commands.rs](mhm/src-tauri/src/commands.rs)

Thêm 2 Tauri commands:

1. **`generate_invoice(booking_id)`** — Tạo invoice record từ booking data:
   - Query booking + guest + room + settings (hotel info)
   - Generate invoice number `INV-YYYYMMDD-XXX`
   - Lấy pricing breakdown từ `pricing_snapshot` trên booking (nếu có) hoặc tính lại
   - Insert vào `invoices` table
   - Return `InvoiceData`

2. **`get_invoice(booking_id)`** — Lấy invoice đã tạo:
   - Query từ `invoices` table
   - Return `InvoiceData` (hoặc None)

**Default policy text:**
```
• Check-in: 14:00 | Check-out: 12:00
• Hủy trước 24h: hoàn cọc 100%
• Hủy trong 24h: giữ cọc 50%
• Không hoàn cọc nếu không đến (No-show)
```

---

#### [MODIFY] [lib.rs](mhm/src-tauri/src/lib.rs)

Register 2 commands mới: `generate_invoice`, `get_invoice`.

---

#### [MODIFY] [gateway/tools.rs](mhm/src-tauri/src/gateway/tools.rs)

Thêm MCP tool `get_invoice` — LLM gọi để lấy invoice text format:

```
get_invoice(booking_id) → text formatted invoice (cho LLM gửi qua Zalo/chat)
```

---

### Frontend — React PDF

---

#### Install dependency

```bash
npm install @react-pdf/renderer
```

---

#### [NEW] [InvoicePDF.tsx](mhm/src/components/InvoicePDF.tsx)

React PDF component dùng `@react-pdf/renderer`:

- **Font**: [Inter](https://fonts.google.com/specimen/Inter) — chuẩn enterprise SaaS, hỗ trợ VN tốt. Register via `Font.register()` từ Google Fonts CDN.
- **Layout**:
  ```
  ┌─────────────────────────────────────┐
  │  [Hotel Name]                       │
  │  [Address] · [Phone]               │
  ├─────────────────────────────────────┤
  │  XÁC NHẬN ĐẶT PHÒNG               │
  │  Số: INV-20260317-001              │
  │  Ngày: 17/03/2026                  │
  ├─────────────────────────────────────┤
  │  Khách: Nguyễn Văn A               │
  │  SĐT: 0912-345-678                │
  ├─────────────────────────────────────┤
  │  Phòng    │ Deluxe 301              │
  │  Loại     │ Deluxe                  │
  │  Check-in │ 17/03/2026              │
  │  Check-out│ 19/03/2026              │
  │  Số đêm   │ 2                       │
  ├─────────────────────────────────────┤
  │  CHI TIẾT GIÁ                      │
  │  2 đêm × 400,000đ    800,000đ     │
  │  Phụ thu cuối tuần      80,000đ     │
  │  ─────────────────────────         │
  │  Tổng cộng:            880,000đ    │
  │  Đã cọc:              200,000đ     │
  │  CÒN LẠI:             680,000đ    │
  ├─────────────────────────────────────┤
  │  📋 Chính sách:                     │
  │  • Check-in: 14:00 | Check-out...  │
  └─────────────────────────────────────┘
  ```
- Color scheme: Navy header + neutral body — chuẩn business document
- Size: A4

---

#### [NEW] [InvoiceDialog.tsx](mhm/src/components/InvoiceDialog.tsx)

Dialog component:
- Nút **"Tải PDF"** — `pdf(document).toBlob()` → save via Tauri `fs` API hoặc browser download
- Nút **"In"** — open PDF in new window → print
- Wrap `<PDFViewer>` cho preview inline

---

#### [MODIFY] [ReservationSheet.tsx](mhm/src/components/ReservationSheet.tsx)

Thêm nút **"📄 Invoice"** trong booking detail → mở `InvoiceDialog`:
- Khi click: gọi `generate_invoice(booking_id)` nếu chưa có, hoặc `get_invoice()` nếu đã tạo
- Truyền `InvoiceData` vào `InvoicePDF` component

---

#### [MODIFY] [CheckinSheet.tsx](mhm/src/components/CheckinSheet.tsx)

Thêm nút **"📄 Invoice"** tương tự trong active booking detail.

---

## Verification Plan

### Automated Tests

1. **Rust unit test** — test `generate_invoice` logic:
   ```bash
   cd mhm/src-tauri && cargo test invoice
   ```
   Test cases: invoice number generation, pricing breakdown serialization, default policy text.

2. **Tauri build** — verify Rust compiles:
   ```bash
   cd mhm/src-tauri && cargo check
   ```

3. **Vite build** — verify frontend compiles with new dependency:
   ```bash
   cd mhm && npm run build
   ```

### Manual Verification

> Anh test bằng cách:
> 1. Mở app → Reservations → Tạo 1 reservation mới
> 2. Click vào reservation → Click nút "📄 Invoice"
> 3. Xem preview PDF → kiểm tra layout, giá, thông tin
> 4. Click "Tải PDF" → mở file PDF, kiểm tra text searchable
