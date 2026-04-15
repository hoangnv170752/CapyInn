Dưới đây là bản **Frontend UI/UX Redesign Blueprint** được biên soạn chi tiết dưới chuẩn Markdown. Em đã trích xuất toàn bộ "tinh hoa" thị giác từ 3 bức ảnh UI đối thủ (phong cách Reservo) và ráp nối trực tiếp vào luồng nghiệp vụ của app **MHM (Mini Hotel Manager - 10 phòng)** của anh.

Bản thiết kế này đặc biệt lưu ý đến môi trường chạy Native Desktop (macOS/Tauri) để mang lại trải nghiệm mượt mà nhất.

Anh có thể copy toàn bộ nội dung trong khung code dưới đây, lưu thành file `frontend-redesign.md` và bỏ vào thư mục tài liệu của dự án để team Frontend bám sát thi công:

---

```markdown
# 🎨 MHM App — Frontend UI/UX Redesign Blueprint

**Phiên bản:** 1.0
**Mục tiêu:** Nâng cấp toàn diện giao diện MHM từ dạng tool quản lý cơ bản lên chuẩn **Premium Modern B2B SaaS** (Sang trọng, phẳng, không gian mở, tinh tế). Tối ưu hóa trải nghiệm thị giác trên màn hình Retina của macOS.
**Tech Stack:** React 18 + TypeScript + Tailwind CSS + shadcn/ui + Recharts.
**Reference UI:** Reservo PMS Design System.

---

## 1. Triết lý Thiết kế Cốt lõi (Core Design Principles)

Để đạt được độ "Wow" và đẳng cấp như bản thiết kế mẫu, toàn bộ UI phải tuân thủ nghiêm ngặt 4 nguyên tắc thị giác sau:

1. **Nền phân tầng (Layered Backgrounds):** Tuyệt đối **KHÔNG** dùng màu trắng tinh (`#FFFFFF`) làm nền tổng thể của ứng dụng. Nền tổng phải là màu xám pha xanh cực nhạt (`bg-slate-50` hoặc `#F8FAFC`). Màu trắng tinh chỉ được dùng cho các khối Card nội dung nổi lên trên.
2. **Bo góc siêu lớn (Massive Border Radius):** Từ bỏ hoàn toàn các góc vuông sắc cạnh thô cứng.
   - Các Card bọc ngoài (Widget): Bo góc `24px` (`rounded-3xl`).
   - Nút bấm, Input, Badge, Thẻ phòng: Bo góc `12-16px` (`rounded-xl` hoặc `rounded-2xl`).
3. **Đổ bóng tản mờ (Soft Drop-shadows):** Hạn chế tối đa dùng viền (border) nét cứng màu đen/xám đậm để phân chia khối. Thay vào đó, dùng hiệu ứng đổ bóng với độ lan tỏa rộng và độ mờ (opacity) cực thấp (2-5%) để tạo cảm giác các khối đang lơ lửng (Floating effect).
4. **Màu Trạng thái Pastel (Pastel Status Colors):** Các màu báo trạng thái (Trống, Đang ở, Cần dọn, Còn nợ) không dùng màu nguyên bản gắt (như Đỏ cờ, Xanh lá cây đậm). Phải dùng hệ màu Pastel: Nền nhạt + Chữ đậm cùng tone.

---

## 2. Thiết lập Cốt lõi (Tailwind CSS Config)

Ghi đè cấu hình mặc định trong `tailwind.config.js` để khóa chặt bảng màu và thông số bo góc/đổ bóng khớp với UI mẫu:

```javascript
/** @type {import('tailwindcss').Config} */
module.exports = {
  // ... (giữ nguyên content paths)
  theme: {
    extend: {
      fontFamily: {
        // Cài đặt Google Font có nét bo tròn hiện đại, dễ đọc số liệu
        sans: ['"Plus Jakarta Sans"', '"Inter"', 'sans-serif'], 
      },
      colors: {
        brand: {
          bg: '#F8FAFC',        // Nền tổng App (Slate 50)
          surface: '#FFFFFF',   // Nền của các widget/card
          primary: '#2563EB',   // Xanh dương chủ đạo (Nút CTA chính)
          text: '#0F172A',      // Chữ tiêu đề (Slate 900)
          muted: '#64748B',     // Chữ phụ, ghi chú (Slate 500)
        },
        // Bảng màu trạng thái phòng/booking (Hệ Pastel)
        status: {
          paid:     { bg: '#EFF6FF', text: '#1D4ED8', border: '#60A5FA' }, // Đã thanh toán (Xanh dương)
          unpaid:   { bg: '#FFF7ED', text: '#C2410C', border: '#FDBA74' }, // Còn nợ (Cam)
          partPaid: { bg: '#FDF2F8', text: '#BE185D', border: '#F472B6' }, // Trả 1 phần (Hồng)
          vacant:   { bg: '#F1F5F9', text: '#475569', border: '#CBD5E1' }, // Trống (Xám)
        }
      },
      borderRadius: {
        'lg': '0.75rem',   // 12px - Input, Thẻ nhỏ
        'xl': '1rem',      // 16px - Nút bấm, Cục booking trên Timeline
        '2xl': '1.5rem',   // 24px - Popup, Dialog
        '3xl': '2rem',     // 32px - Card nội dung bao ngoài
      },
      boxShadow: {
        // Bóng mờ đặc trưng làm nên sự sang trọng
        'soft': '0 10px 40px -10px rgba(0,0,0,0.04)', 
        'float': '0 20px 50px -10px rgba(0,0,0,0.08)', // Dùng cho Panel trượt
      }
    },
  },
}

```

Thêm CSS ẩn thanh cuộn thô của macOS/Trình duyệt vào `globals.css`:

```css
/* Tối ưu thanh cuộn (Custom Scrollbar) */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #cbd5e1; border-radius: 10px; }
::-webkit-scrollbar-thumb:hover { background: #94a3b8; }

```

---

## 3. Kiến trúc App Shell (Layout Tổng thể)

Chia layout làm 2 phần dùng CSS Flexbox. Đặc biệt chú ý tính năng kéo thả cửa sổ đặc thù của macOS Tauri bằng class `data-tauri-drag-region`.

```tsx
// src/components/layout/AppLayout.tsx
export default function AppLayout({ children }) {
  return (
    {/* select-none để chống bôi đen text bậy bạ, tạo cảm giác Native App */}
    <div className="flex h-screen w-screen bg-brand-bg font-sans text-brand-text overflow-hidden select-none">
      
      {/* 1. SIDEBAR TRÁI: Rộng cố định 260px, Nền trắng */}
      <aside className="w-[260px] bg-white border-r border-slate-100 flex flex-col p-6 z-20 shrink-0">
        <div className="flex items-center gap-3 mb-10">
          <div className="w-8 h-8 rounded-lg bg-brand-primary text-white flex items-center justify-center font-bold">M</div>
          <span className="font-bold text-xl tracking-tight">MHM Hotel</span>
        </div>
        
        {/* Navigation Menu */}
        <nav className="flex flex-col gap-2">
           {/* Dùng Button variant="ghost" của shadcn, căn trái, bo góc xl */}
           <NavItem icon={<Home />} label="Dashboard" active />
           <NavItem icon={<Calendar />} label="Timeline" />
        </nav>
      </aside>

      {/* 2. MAIN CONTENT AREA */}
      <main className="flex-1 flex flex-col h-full relative min-w-0">
        
        {/* HEADER: Trong suốt mờ (Backdrop Blur), Khu vực kéo thả cửa sổ App */}
        {/* data-tauri-drag-region: Cực kỳ quan trọng để user dùng chuột nắm kéo app trên Mac */}
        <header className="h-[88px] flex items-center justify-between px-10 bg-brand-bg/80 backdrop-blur-md sticky top-0 z-10 data-tauri-drag-region shrink-0">
          <div className="pointer-events-none">
            <h1 className="text-2xl font-bold tracking-tight">Thống kê</h1>
            <p className="text-sm text-brand-muted">Thứ Bảy, 14 Tháng 3, 2026</p>
          </div>
          
          <div className="flex items-center gap-4 pointer-events-auto">
             <Badge className="bg-green-50 text-green-700 border-0 rounded-full py-1.5 px-3">
               🟢 Scanner Ready
             </Badge>
             <Button className="rounded-xl bg-brand-primary shadow-soft hover:shadow-float transition-all px-6 py-5">
               + Khách mới (Manual)
             </Button>
          </div>
        </header>

        {/* 3. VÙNG CUỘN NỘI DUNG (Scrollable Area) */}
        <div className="flex-1 overflow-y-auto px-10 pb-10">
          {children}
        </div>
        
      </main>
    </div>
  );
}

```

---

## 4. Triển khai Layout theo Từng Màn Hình

### 4.1. Màn hình Dashboard (Tham chiếu: Ảnh 1 - Bento Grid)

Sử dụng **CSS Grid (`grid-cols-12`)** để bố cục các khối Card màu trắng xếp khít nhau một cách gọn gàng.
Tất cả các widget bọc trong thẻ: `<div className="bg-white rounded-3xl shadow-soft p-6">`

* **Widget Analytics (Cột 8 - Trái):** Dùng thư viện `Recharts` (`<AreaChart>`).
* Bắt buộc set `type="monotone"` trong thẻ `<Area>` để tạo đường cong uốn lượn mềm mại.
* Tắt các đường lưới dọc thừa thãi, chỉ giữ lưới ngang màu siêu nhạt (`#F1F5F9`).


* **Widget Sơ đồ 10 Phòng (Cột 4 - Phải):** Lưới `grid-cols-2 gap-3`. Mỗi phòng là 1 thẻ vuông bo góc `rounded-xl`, bôi màu nền theo state (Trống/Có khách/Cần dọn).
* **Widget Danh sách khách (Cột 8 - Dưới):** Dùng `<Table>` của shadcn.
* **Quy tắc sống còn:** Bỏ toàn bộ viền dọc (`border-x`). Chỉ giữ đường kẻ ngang dưới cùng cực mỏng cho mỗi hàng (`border-b border-slate-50`). Header in hoa nhẹ, màu xám (`text-xs uppercase text-slate-400 font-semibold`).



### 4.2. Màn hình Timeline Đặt phòng (Tham chiếu: Ảnh 2 - Vũ khí vận hành)

Màn hình này thay thế hoàn toàn sổ tay, giúp Lễ tân nhìn thấu luồng khách. Không dùng thư viện Gantt nặng nề, tự code bằng CSS Grid.

* **Cấu trúc Lưới:** Trục Y là danh sách 10 phòng (Rộng cố định `150px`). Trục X là thanh thời gian (Mỗi cột 1 ngày, rộng ~`100px`).
* **Booking Pills (Cục Booking):** Hiển thị dạng các khối nằm ngang.
* *Thực thi:* Dùng CSS `position: absolute` bám lên lưới.
* *Style:* Bo góc `rounded-xl`, đổ `shadow-sm`, có hiệu ứng nảy khi hover (`hover:-translate-y-1 transition-transform`).
* *Màu sắc:* Nền pastel tuỳ trạng thái thanh toán (Map với `colors.status`).
* *Điểm nhấn thị giác:* Thêm một vạch viền dọc ở cạnh trái (`border-l-4 border-status-paid-border`) để cục booking trông sắc nét y hệt ảnh 2.



### 4.3. Luồng Quét CCCD OCR (Tham chiếu: Ảnh 3 - Slide-over Panel)

Tuyệt đối KHÔNG dùng Popup Modal hiện ra giữa màn hình che khuất Sơ đồ phòng. MHM sẽ dùng tính năng **Bảng trượt từ bên phải** mô phỏng khung AI Assistant.

**Kịch bản UI/UX Mượt mà (Seamless Flow):**

1. Lễ tân đang ở màn hình Timeline (Ảnh 2), đút thẻ CCCD vào máy scan Canon.
2. *Rust File Watcher* bắt được file, chạy MNN OCR ngầm (~300ms).
3. App tự động trượt một Panel từ mép phải màn hình ra. Dùng component `<Sheet side="right" className="w-[450px] shadow-float border-0">` của shadcn.
4. **Bố cục bên trong Panel OCR:**
* **Top:** Hiển thị trực tiếp tấm ảnh thẻ CCCD vừa được crop gọn gàng (bo góc `rounded-xl`) để lễ tân đối chiếu bằng mắt.
* **Middle:** Form nhập liệu gồm các ô `<Input>` (nền xám `bg-slate-50`, không viền cứng) đã được điền sẵn Text bóc từ OCR (Họ Tên, ID, Ngày sinh).
* **Bottom:** Dropdown `<Select>` chọn phòng trống và Nút CTA bự `"Xác nhận & Nhận phòng"`.


5. **Điểm 10 UX:** Nhờ Panel nằm gọn bên phải, nửa trái màn hình vẫn hiển thị Sơ đồ phòng. Lễ tân không bị "mù" thông tin, mắt liếc trái xem phòng nào trống, tay thao tác chuột bên phải để assign khách ngay lập tức.

---

## 5. Checklist Setup `shadcn/ui` Components

Chạy các lệnh sau trong Terminal để kéo chính xác các "vật liệu" về build UI:

```bash
npx shadcn-ui@latest add button card table badge sheet input select label

```

**Tùy biến Component (Overrides):**
Sau khi cài, hãy vào thư mục `components/ui/` sửa lại các class mặc định để khớp hoàn toàn với Design:

* `button.tsx`: Đổi `rounded-md` thành `rounded-xl`.
* `card.tsx`: Xoá bỏ border (`border`), thêm `shadow-soft rounded-3xl bg-white`.
* `input.tsx`: Chuyển nền thành xám nhạt (`bg-slate-50`), xóa viền (`border-transparent`). Khi `:focus` mới hiện viền màu Primary.
* `badge.tsx`: Bo góc `rounded-full` hoặc `rounded-lg`, loại bỏ border mặc định, truyền màu nền Pastel.

```

```