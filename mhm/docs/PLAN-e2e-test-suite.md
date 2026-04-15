# PLAN: E2E Test Suite cho MHM Hotel Manager

> **Project Type:** Desktop (Tauri 2 + React 19 + Rust + SQLite)
> **Agent:** `@project-planner` + `@webapp-testing` skill
> **Mode:** PLANNING ONLY — không viết code

---

## 1. Overview

MHM là Tauri desktop app — KHÔNG chạy trên browser thông thường. Điều này ảnh hưởng trực tiếp đến việc chọn testing framework:

| Approach | Phù hợp? | Lý do |
|----------|-----------|-------|
| **Playwright (browser)** | ⚠️ Hạn chế | Chỉ test được UI qua Vite dev server, KHÔNG test được Tauri commands (invoke) |
| **Vitest + React Testing Library** | ✅ Tốt nhất | Mock Tauri invoke, test UI + business logic end-to-end trong JSDOM |
| **Tauri E2E (WebDriver)** | 🔴 Phức tạp | Cần build app trước, setup WebDriver, chậm |

### Chiến lược đề xuất: **Vitest + RTL + MSW/Mock**

Test toàn bộ user flows bằng cách mock Tauri `invoke()` — giả lập tất cả backend commands, test UI từ đầu đến cuối như user thực tế sử dụng.

---

## 2. Success Criteria

| # | Tiêu chí | Đo lường |
|---|----------|----------|
| 1 | Cover 100% critical user flows | 8 test suites, 40+ test cases |
| 2 | Tất cả tests pass | `npm run test` exit 0 |
| 3 | Thời gian chạy < 30s | Fast feedback loop |
| 4 | Không flaky tests | Deterministic mock data |
| 5 | CI-ready | Scripts chạy headless |

---

## 3. Tech Stack

| Tool | Vai trò |
|------|---------|
| **Vitest** | Test runner (Vite-native, nhanh) |
| **@testing-library/react** | Render components, query DOM |
| **@testing-library/user-event** | Mô phỏng tương tác user (click, type) |
| **@testing-library/jest-dom** | Custom matchers (toBeInTheDocument, etc) |
| **msw** *(optional)* | Mock network nếu cần |

---

## 4. File Structure

```
mhm/
├── src/
│   └── __mocks__/
│       └── tauri.ts              ← Mock @tauri-apps/api/core (invoke)
├── tests/
│   ├── setup.ts                  ← Global test setup
│   ├── helpers/
│   │   ├── mock-data.ts          ← Factory functions cho Room, Guest, Booking...
│   │   └── render-app.tsx        ← Custom render helper (providers, stores)
│   └── e2e/
│       ├── 01-login.test.tsx           ← PIN login flow
│       ├── 02-dashboard.test.tsx       ← Dashboard load, room status
│       ├── 03-checkin.test.tsx         ← Check-in flow (mở sheet, fill form, submit)
│       ├── 04-room-detail.test.tsx     ← Room detail, extend stay, copy info
│       ├── 05-checkout.test.tsx        ← Check-out flow
│       ├── 06-housekeeping.test.tsx    ← Housekeeping status updates
│       ├── 07-analytics.test.tsx       ← Analytics data display
│       ├── 08-settings.test.tsx        ← Settings save/load
│       ├── 09-reservations.test.tsx    ← Booking list, filter
│       ├── 10-guests.test.tsx          ← Guest list, search, profile
│       ├── 11-night-audit.test.tsx     ← Night audit flow
│       └── 12-navigation.test.tsx      ← Sidebar nav, collapse, logout
├── vitest.config.ts              ← Vitest configuration
└── package.json                  ← Test scripts
```

---

## 5. Task Breakdown

### Phase 1: Setup Testing Infrastructure

#### Task 1.1: Install dependencies
- **INPUT:** `package.json` hiện tại
- **OUTPUT:** Dev dependencies installed
- **VERIFY:** `npx vitest --version` trả về version

```bash
npm install -D vitest @testing-library/react @testing-library/user-event @testing-library/jest-dom jsdom @types/testing-library__jest-dom
```

#### Task 1.2: Configure Vitest
- **INPUT:** `vite.config.ts`, `tsconfig.json`
- **OUTPUT:** `vitest.config.ts` + `tests/setup.ts`
- **VERIFY:** `npx vitest run --reporter=verbose` chạy được (0 tests found)

#### Task 1.3: Create Tauri Mock
- **INPUT:** Tất cả 39 Tauri commands scan được
- **OUTPUT:** `src/__mocks__/tauri.ts` — mock `invoke()` theo command name
- **VERIFY:** Import mock thành công trong test

#### Task 1.4: Create Test Helpers
- **INPUT:** TypeScript interfaces (Room, Guest, Booking...)
- **OUTPUT:** `tests/helpers/mock-data.ts` + `tests/helpers/render-app.tsx`
- **VERIFY:** Helper functions type-check OK

---

### Phase 2: Core Business Flow Tests

#### Task 2.1: `01-login.test.tsx` — Authentication
| Test Case | Mô tả |
|-----------|--------|
| renders PIN pad | Hiển thị numpad 0-9 + backspace |
| login success | Nhập 4 digit → invoke login → redirect dashboard |
| login failure | PIN sai → shake animation, error message |
| auto-submit | Khi đủ 4 digit tự động submit |
| backspace works | Xóa digit cuối |

#### Task 2.2: `12-navigation.test.tsx` — Navigation & Layout
| Test Case | Mô tả |
|-----------|--------|
| sidebar renders all nav items | 7 items: Dashboard, Reservations, Rooms, Guests, Housekeeping, Analytics, Night Audit + Settings |
| clicking nav changes page | Click "Rooms" → hiển thị Rooms page |
| sidebar collapse/expand | Click chevron → sidebar thu gọn |
| logout button works | Click logout → quay lại login screen |
| hotel name from settings | Hiển thị tên khách sạn từ settings |

#### Task 2.3: `02-dashboard.test.tsx` — Dashboard
| Test Case | Mô tả |
|-----------|--------|
| loads rooms and stats | invoke get_rooms, get_dashboard_stats |
| displays room cards | 10 rooms render đúng |
| room status colors | vacant=green, occupied=red, cleaning=yellow |
| stats overview | Hiện total rooms, occupied, vacant, revenue |
| click room → detail | Click room card → navigate to RoomDetail |

#### Task 2.4: `03-checkin.test.tsx` — Check-in Flow
| Test Case | Mô tả |
|-----------|--------|
| open checkin sheet | Click "Khách mới" → sheet opens |
| select room | Chọn phòng từ dropdown |
| fill guest info | Nhập tên, CCCD, ngày sinh |
| set nights | Nhập số đêm |
| submit check-in | Gọi invoke check_in → room status → occupied |
| validation | Thiếu field → hiện error |

#### Task 2.5: `04-room-detail.test.tsx` — Room Detail
| Test Case | Mô tả |
|-----------|--------|
| render room info | Thông tin phòng, type, floor |
| render booking info | Guest name, check-in/out dates, nights |
| vacant room | Không có booking info |
| extend stay button | Click → invoke extend_stay |
| copy stay info | Click → invoke get_stay_info_text |

#### Task 2.6: `05-checkout.test.tsx` — Check-out Flow
| Test Case | Mô tả |
|-----------|--------|
| checkout button visible | Chỉ khi room occupied |
| confirm dialog | Hiện total, paid, remaining |
| submit checkout | invoke check_out → room → cleaning |

---

### Phase 3: Management Features Tests

#### Task 3.1: `06-housekeeping.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| load tasks | invoke get_housekeeping_tasks |
| display task list | Room, status, timestamp |
| update status | needs_cleaning → cleaning → clean |

#### Task 3.2: `07-analytics.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| load analytics data | invoke get_analytics |
| period filter | today, week, month |
| charts render | Revenue chart, occupancy |

#### Task 3.3: `08-settings.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| load hotel info | invoke get_settings → pre-fill |
| save hotel info | Edit → save → invoke save_settings |
| load checkin rules | get_settings checkin_rules |
| pricing rules | get_pricing_rules, save_pricing_rule |

#### Task 3.4: `09-reservations.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| load bookings | invoke get_all_bookings |
| filter by status | active, checked_out |
| booking details | Room, guest, dates, amount |

#### Task 3.5: `10-guests.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| load guest list | invoke get_all_guests |
| search guests | Type → filter results |
| guest profile | Click → GuestProfileSheet |

#### Task 3.6: `11-night-audit.test.tsx`
| Test Case | Mô tả |
|-----------|--------|
| run night audit | invoke run_night_audit |
| audit logs | invoke get_audit_logs |
| audit results display | Summary, discrepancies |

---

## 6. Tauri Mock Strategy

Mock **tất cả** `@tauri-apps/api/core` invoke calls. Mỗi command trả về mock data tương ứng:

```typescript
// Ví dụ mock strategy
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((command: string, args?: any) => {
    switch (command) {
      case 'get_rooms': return Promise.resolve(mockRooms);
      case 'login': return args.req.pin === '1234'
        ? Promise.resolve({ user: mockUser })
        : Promise.reject('Invalid PIN');
      case 'get_dashboard_stats': return Promise.resolve(mockStats);
      // ... 36 more commands
    }
  })
}));
```

Cũng cần mock:
- `@tauri-apps/api/event` → `listen()` function
- `localStorage` → sidebar collapse state

---

## 7. Phase X: Verification

### Automated
```bash
# Chạy toàn bộ test suite
npm run test

# Chạy với coverage
npm run test:coverage

# Chạy 1 file cụ thể
npx vitest run tests/e2e/01-login.test.tsx
```

### Pass Criteria
- [ ] Tất cả tests PASS
- [ ] Coverage > 70% trên critical paths
- [ ] Không có flaky tests (chạy 3 lần liên tiếp)
- [ ] `npm run build` vẫn pass (tests không ảnh hưởng build)

---

## 8. Thời gian ước tính

| Phase | Tasks | Est. time |
|-------|-------|-----------|
| Phase 1 | Setup (4 tasks) | ~20 min |
| Phase 2 | Core flows (6 suites) | ~60 min |
| Phase 3 | Management (6 suites) | ~45 min |
| Phase X | Verify | ~10 min |
| **Total** | **16 tasks, 12 test files** | **~2.5 hours** |

---

*Plan created by `@project-planner` + `@webapp-testing` skill*
