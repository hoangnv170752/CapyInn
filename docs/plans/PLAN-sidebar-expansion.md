# PLAN: Sidebar Navigation Expansion — MHM Hotel

> **Goal:** Expand the 3-tab sidebar (Dashboard, Timeline, Housekeeping) into a full hotel management suite with **Rooms**, **Reservations**, **Guests**, **Analytics**, and **Settings** tabs — matching the Reservo competitor's depth while maintaining MHM's premium design language.
>
> **Note:** AI Assistant / Command Palette is **deferred** to a future phase when the core app is complete. Agentic AI sẽ giao tiếp qua gateway riêng.

---

## Current State (What We Have)

| Layer | Assets |
|-------|--------|
| **DB Tables** | `rooms`, `guests`, `bookings`, `booking_guests`, `transactions`, `expenses`, `housekeeping` |
| **Tauri Commands** | `get_rooms`, `get_dashboard_stats`, `check_in`, `get_room_detail`, `check_out`, `extend_stay`, `get_housekeeping_tasks`, `update_housekeeping`, `create_expense`, `get_expenses`, `get_revenue_stats`, `get_stay_info_text`, `scan_image` |
| **Frontend Pages** | `Dashboard.tsx`, `Timeline.tsx`, `Housekeeping.tsx`, `RoomDetail.tsx`, `Statistics.tsx` (unused) |
| **Seed Data** | 10 rooms (1A-5B), 5 floors, 2 types (deluxe/standard) |

> [!IMPORTANT]
> Most backend commands already exist. This plan is **80% frontend**, with a few new Rust commands needed for Guests and Analytics.

---

## Phase 1: Rooms Tab — Floor Map Visual *(P0)*

### What it does
Full-screen floor-plan view showing all rooms as color-coded cards grouped by floor, with click-to-view detail.

### Backend Changes
**None** — `get_rooms` and `get_room_detail` already exist.

### Frontend Changes

#### [NEW] `src/pages/Rooms.tsx`
- Tab bar at top for floors (Floor 1 / Floor 2 / Floor 3 / ...)
- Grid layout of `RoomCard` components per floor
- Color-code: Vacant (green), Occupied (blue), Cleaning (amber), Booked (purple)
- Click on room → opens existing `RoomDetail` as a **Sheet slide-over** (reuse Shadcn Sheet pattern from OCR panel)
- Stats bar at bottom: `X Vacant · Y Occupied · Z Cleaning`

#### [MODIFY] `src/components/RoomCard.tsx`
- Add `onClick` prop to navigate to Room detail
- Add subtle hover animation (`scale-[1.02]`)

#### [MODIFY] `src/App.tsx`
- Add `"rooms"` to `activeTab` union type
- Add Rooms nav item to sidebar
- Route to `<Rooms />` component

#### [MODIFY] `src/stores/useHotelStore.ts`
- Extend `activeTab` type with all new tab keys

---

## Phase 2: Reservations Tab — Gantt Timeline *(P1)*

> **Decision:** Merge existing `Timeline.tsx` into Reservations tab (matching Reservo's Reservations page which IS a Gantt chart). Remove separate Timeline tab.

### What it does
Full Gantt-chart view of all bookings grouped by room type (Standard / Deluxe), with status filter badges and search.

### Backend Changes

#### [MODIFY] `src-tauri/src/commands.rs`
- **New command:** `get_all_bookings(filter: BookingFilter) -> Vec<BookingWithGuest>`
  - Joins `bookings` + `guests` table to return guest name alongside booking
  - Filter by: `status` (all/active/upcoming/completed), `date_range`, `room_id`
  - Sorted by `check_in_at DESC`

#### [MODIFY] `src-tauri/src/models.rs`
- Add `BookingFilter` struct
- Add `BookingWithGuest` response struct

#### [MODIFY] `src-tauri/src/lib.rs`
- Register new `get_all_bookings` command

### Frontend Changes

#### [MODIFY] `src/pages/Reservations.tsx` (evolve from Timeline.tsx)
- **Status filter badges** at top: `Occupied 20` (blue), `Check-in/Check-out 09` (amber), `Reserved 02` (purple) — matching Reservo exactly
- Search bar + Filter button
- Gantt chart grouped by room type sections (Standard / Deluxe) instead of flat list
- Each booking bar shows: guest name, source (Booking/Direct), payment badge (Paid/Unpaid/Part-paid), check-in time
- Today indicator line (vertical blue line)
- Click booking bar → opens detail Sheet

#### [DELETE] `src/pages/Timeline.tsx`
- Functionality absorbed into Reservations

#### [MODIFY] `src/stores/useHotelStore.ts`
- Add `bookings: BookingWithGuest[]` state
- Add `fetchBookings(filter?)` action

---

## Phase 3: Guests Tab — Guest Directory + Profile *(P2)*

### What it does
CRM-lite: searchable guest list with profile drawer showing stay history.

### Backend Changes

#### [MODIFY] `src-tauri/src/commands.rs`
- **New command:** `get_all_guests(search?: String) -> Vec<GuestSummary>`
  - Query `guests` table with optional `LIKE` search on `full_name` or `doc_number`
  - Aggregate: total stays count, total spent, last visit date (from `bookings` JOIN)
- **New command:** `get_guest_history(guest_id: String) -> GuestHistory`
  - Returns full guest info + list of past bookings with room/dates/amounts

#### [MODIFY] `src-tauri/src/models.rs`
- Add `GuestSummary` struct (id, name, doc_number, total_stays, total_spent, last_visit)
- Add `GuestHistory` struct

#### [MODIFY] `src-tauri/src/lib.rs`
- Register `get_all_guests`, `get_guest_history`

### Frontend Changes

#### [NEW] `src/pages/Guests.tsx`
- Search bar with debounced search
- Table: Name, CCCD, Quốc tịch, Lần ở, Tổng chi tiêu, Lần cuối
- VIP badge (≥5 stays), Returning badge (≥2 stays)
- Click row → opens `GuestProfileSheet` (Sheet slide-over)

#### [NEW] `src/components/GuestProfileSheet.tsx`
- Guest info section (từ OCR data)
- Stay history timeline (vertical timeline of past bookings)
- Total spending metric

#### [MODIFY] `src/stores/useHotelStore.ts`
- Add `guests: GuestSummary[]` state
- Add `fetchGuests(search?)`, `fetchGuestHistory(id)` actions

---

## Phase 4: Analytics Tab — Multi-Chart Dashboard *(P3)*

### What it does
Business intelligence page with revenue charts, occupancy metrics, and expense tracking.

### Backend Changes

#### [MODIFY] `src-tauri/src/commands.rs`
- **New command:** `get_analytics(period: String) -> AnalyticsData`
  - Aggregates across bookings, expenses, rooms
  - Returns: revenue, occupancy_rate, ADR, RevPAR, daily breakdown, revenue by source, expenses by category, top rooms
  - `period`: "7d" | "30d" | "90d"

#### [MODIFY] `src-tauri/src/models.rs`
- Add `AnalyticsData` struct with sub-structs

#### [MODIFY] `src-tauri/src/lib.rs`
- Register `get_analytics`

### Frontend Changes

#### [NEW] `src/pages/Analytics.tsx`
- **Row 1:** 4 KPI StatCards — Revenue, Occupancy %, ADR, RevPAR
- **Row 2:** Area chart (revenue over time) + Donut chart (room type split)
- **Row 3:** Horizontal bar (revenue by source) + Bar chart (top 5 rooms)
- **Row 4:** Expense table (from existing `get_expenses` command)
- Period toggle: 7D / 30D / 90D

#### [MODIFY] `src/stores/useHotelStore.ts`
- Add `analytics: AnalyticsData | null` state
- Add `fetchAnalytics(period)` action

---

## Phase 5: Settings Tab *(P4)*

### What it does
Configuration page with vertical sub-navigation.

### Backend Changes

#### [MODIFY] `src-tauri/src/commands.rs`
- **New command:** `update_room(room_id, updates) -> Room` — edit room price/type
- **New command:** `export_data(format: "csv") -> String` — returns file path of exported CSV

### Frontend Changes

#### [NEW] `src/pages/Settings.tsx`
- Vertical tab layout (left sub-nav + right content area)
- **Sections:**
  - Hotel Info (name, address — stored in localStorage for now)
  - Room Management (editable table of rooms with inline price editing)
  - Check-in Rules (default hours, checkout time)
  - OCR Config (scan folder path, auto-scan toggle)
  - Appearance (light/dark mode toggle, language VI/EN)
  - Data (export CSV button, backup database)

---

## Phase 6: Dashboard Enrichment *(P5)*

> **Purpose:** Fill the empty spaces on Dashboard to match Reservo's density.

### What it does
Add 3 new widgets to Dashboard: Activity Feed, Expense Breakdown, and Hotel Promotion Card.

### Backend Changes

#### [MODIFY] `src-tauri/src/commands.rs`
- **New command:** `get_recent_activity(limit: i32) -> Vec<ActivityItem>`
  - Query recent check-ins, check-outs, and housekeeping events
  - Each item: timestamp, action type (check-in/check-out/cleaning), description, room_id
  - Sorted by `created_at DESC`, limited

#### [MODIFY] `src-tauri/src/models.rs`
- Add `ActivityItem` struct

### Frontend Changes

#### [MODIFY] `src/pages/Dashboard.tsx` — Add 3 new widgets

**Widget 1: Today's Activity Feed (replace empty right column)**
- Vertical list of today's activities with timestamps
- Icons per type: 🟢 Check-in, 🔴 Check-out, 🧹 Cleaning
- Format: `14:00 — Check-in Nguyễn Văn A → 1A`
- Auto-refreshes, scrollable, max 10 items
- "Xem tất cả" link at bottom

**Widget 2: Expense Category Breakdown (below Guests table)**
- Horizontal progress bars for each expense category
- Each row: category name, progress bar, amount / budget cap
- Categories: Điện, Nước, Nhân viên, Vật tư, Khác
- Reuses existing `get_expenses` command with date filter
- Matches Reservo's "Extra Revenue" widget layout

**Widget 3: Hotel Promotion Card (top right corner)**
- Static card with hotel photo/gradient background
- Rating display: "4.8 — Based on 50 reviews"
- "Discover" link (configurable in Settings)
- Data stored in localStorage, editable from Settings tab

---

## Phase 7: Sidebar Collapse Toggle *(P6)*

### What it does
Add a toggle button at the bottom of the sidebar to collapse it into icon-only mode (~60px wide).

### Backend Changes
**None** — purely frontend UX.

### Frontend Changes

#### [MODIFY] `src/App.tsx`
- Add `isCollapsed` state (persisted in localStorage)
- Toggle button at bottom of sidebar (chevron icon)
- When collapsed: sidebar width = 60px, show only icons with tooltip on hover
- When expanded: sidebar width = 260px (current), show icon + label
- Smooth CSS transition: `transition-all duration-300`

---

## Cross-Cutting Changes

### [MODIFY] `src/App.tsx` — Sidebar Navigation Update (Final)
```
Sidebar items (final order):
─── MAIN ───
  Dashboard        (Home icon)
  Reservations     (Calendar icon)     ← NEW (absorbs Timeline)
  Rooms            (BedDouble icon)    ← NEW
  Guests           (Users icon)        ← NEW

─── MANAGEMENT ───
  Housekeeping     (Sparkles icon)
  Analytics        (BarChart3 icon)    ← NEW

─── SYSTEM ───
  Settings         (Settings icon)     ← NEW

─── BOTTOM ───
  Collapse toggle  (ChevronsLeft icon) ← NEW
```

> **Note:** Timeline tab removed — merged into Reservations.

### [MODIFY] `src/stores/useHotelStore.ts` — Tab Type
```typescript
activeTab: "dashboard" | "detail" | "rooms" | "reservations" | "guests" | "housekeeping" | "analytics" | "settings"
```

---

## Implementation Priority & Effort

| Phase | Feature | Effort | New Backend Commands | New Frontend Files |
|-------|---------|--------|---------------------|-------------------|
| 1 | Rooms (Floor Map) | **Low** | 0 | 1 page |
| 2 | Reservations (Gantt) | **Medium** | 1 | 1 page (evolve Timeline) |
| 3 | Guests (Directory) | **Medium** | 2 | 2 files (page + sheet) |
| 4 | Analytics (Charts) | **Medium-High** | 1 | 1 page |
| 5 | Settings | **Medium** | 2 | 1 page |
| 6 | Dashboard Widgets | **Medium** | 1 | 0 (modify Dashboard) |
| 7 | Sidebar Collapse | **Low** | 0 | 0 (modify App.tsx) |

> [!TIP]
> Phases 1-3 can be done **frontend-first** with mock data, then wired up when backend commands are ready. This allows for rapid visual iteration.

---

## Verification Plan

### Build Check
```bash
cd ./mhm && npm run build
```
Must pass with 0 TypeScript errors.

### Rust Compilation
```bash
cd ./mhm/src-tauri && cargo check
```
Must compile without errors after adding new commands.

### Visual Verification
Run `npm run tauri dev` and manually verify:
1. Sidebar has 7 items + collapse toggle (no more Timeline tab)
2. Each tab navigates to the correct page
3. Rooms page shows floor-grouped room cards with correct status colors
4. Reservations shows Gantt chart with filter badges (Occupied/Check-in/Reserved)
5. Guests search filters correctly, profile sheet opens on click
6. Analytics charts render with real data, period toggle works
7. Settings page sections are navigable
8. Dashboard shows Activity Feed, Expense widget, and Hotel card
9. Sidebar collapse/expand works smoothly

### User Manual Testing
After implementation, the user should:
1. Click each sidebar tab to confirm navigation works
2. On Rooms: click a room card to open the detail sheet
3. On Reservations: verify Gantt chart shows bookings grouped by room type
4. On Guests: search for a guest by name, open profile
5. On Analytics: toggle between 7D/30D/90D periods
6. On Settings: edit a room price and verify it persists
7. On Dashboard: verify activity feed, expense breakdown, and hotel card display
8. Toggle sidebar collapse and verify it remembers state after reload
