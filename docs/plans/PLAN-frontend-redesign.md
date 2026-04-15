# 🎨 MHM Frontend Redesign Project Plan

## Overview
Redesign the frontend of the MHM (Mini Hotel Manager) Tauri Rust application to achieve a premium, modern B2B SaaS look (similar to the Reservo PMS). The design will feature layered backgrounds, massive border radii, soft drop-shadows, and pastel status colors.

## Project Type
WEB (Tauri frontend using React/Next.js)

## Success Criteria
- UI matches the provided high-fidelity screenshots in aesthetics (Wow factor).
- The application feels native on macOS (using `select-none` and `data-tauri-drag-region`).
- Key screens (Dashboard, Timeline, OCR Panel) are fully implemented.
- Shadcn UI components are customized to match the design system.
- All Phase X verifications pass successfully.

## Tech Stack
- **Framework**: React 18 + TypeScript (or Next.js depending on current setup).
- **Styling**: Tailwind CSS.
- **Components**: shadcn/ui.
- **Charts**: Recharts.
- **Desktop Environment**: Tauri (Rust) for macOS compatibility.

## File Structure
- `src/components/layout/AppLayout.tsx`: Native-feeling app shell with sidebar and Mac drag region.
- `src/components/ui/*`: Overridden shadcn components (button, card, input, badge, sheet).
- `src/features/dashboard/*`: Bento grid analytics and room status widgets.
- `src/features/timeline/*`: Custom CSS grid Gantt chart for reservations.
- `src/features/ocr-scanner/*`: Slide-over panel (Sheet) for ID scanning.
- `tailwind.config.js`: Custom theme colors (Pastel), shadows, and border radii.
- `src/styles/globals.css`: Custom scrollbar and base styles.

## Task Breakdown

### 1. Project Setup & Theme Configuration
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`, `tailwind-patterns`
- **INPUT**: `frontend-redesign.md` specs.
- **OUTPUT**: Updated `tailwind.config.js` and `globals.css` with pastel colors, soft shadows, and 24px-32px border radii.
- **VERIFY**: Tailwind compiles successfully; Scrollbars are hidden; Base typography is active.

### 2. Component Customization (shadcn/ui)
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`
- **INPUT**: Base shadcn components (button, card, table, badge, sheet, input, select).
- **OUTPUT**: Overridden components in `src/components/ui/` that match the "massive border radius" design philosophy.
- **VERIFY**: Elements display correctly with the new styling rules.

### 3. App Shell & Layout Implementation
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`
- **INPUT**: Layout mockups from the redesign blueprint.
- **OUTPUT**: Functional `AppLayout.tsx` acting as the application shell.
- **VERIFY**: App renders without horizontal scroll; macOS window dragging (`data-tauri-drag-region`) works; Sidebar navigation is functional.

### 4. Dashboard View Implementation
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`
- **INPUT**: Dashboard reference images (Image 1 & 2).
- **OUTPUT**: Interactive bento-grid dashboard with Recharts analytics and Guest Table.
- **VERIFY**: Grid is responsive; Charts render with smooth curves; Data flows to tables without breaking visual boundaries.

### 5. Timeline (Reservations) View
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`
- **INPUT**: Reservations Gantt reference (Image 1).
- **OUTPUT**: Timeline screen using CSS grid with pastel booking pills.
- **VERIFY**: Scrolling is smooth; Booking pills map accurately to relative dates and rooms.

### 6. OCR Assistant Panel (Slide-over)
- **Agent**: `frontend-specialist`
- **Skill**: `frontend-design`
- **INPUT**: OCR slide-over concept based on the AI Assistant panel (Image 3).
- **OUTPUT**: Functional slide-over panel utilizing the `Sheet` component for ID scanning results.
- **VERIFY**: Panel slides in seamlessly; Form inputs are styled correctly over a light grey background (`bg-slate-50`).

## ✅ PHASE X: VERIFICATION (MANDATORY CHECKS)
- [ ] **Lint & Validate**: Run `npm run lint` and `npx tsc --noEmit` to clear all errors.
- [ ] **Design Rules Check**: No purple/violet hex codes used. No standard/cliché template layouts.
- [ ] **UX & Accessibility**: Run `ux_audit.py` to ensure high contrast for pastel text/background combinations.
- [ ] **Visual Match**: Confirm UI strongly aligns with the 3 target screenshots (Reservo).
- [ ] **Tauri Build Test**: Application builds successfully as a Tauri desktop app without breaking UI features.
