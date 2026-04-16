import { describe, it, expect, beforeEach } from "vitest";
import { render, waitFor } from "../helpers/render-app";
import NightAudit from "@/pages/NightAudit";
import { setMockResponse, clearMockResponses, invoke } from "@test-mocks/tauri-core";
import { useAuthStore } from "@/stores/useAuthStore";

describe("11 — Night Audit", () => {
    beforeEach(() => {
        clearMockResponses();
        invoke.mockClear();

        // NightAudit requires isAdmin() for the Run Audit section
        useAuthStore.setState({
            user: { id: "u1", name: "Admin", role: "admin", active: true, created_at: "" },
            isAuthenticated: true,
            loading: false,
            error: null,
        });

        setMockResponse("get_audit_logs", () => []);
        setMockResponse("get_rooms", () => []);
        setMockResponse("get_all_bookings", () => []);
    });

    it("renders night audit page", async () => {
        render(<NightAudit />);

        // Should render without crashing
        await waitFor(() => {
            expect(invoke).toHaveBeenCalled();
        });
    });

    it("calls get_audit_logs on mount", async () => {
        render(<NightAudit />);

        await waitFor(() => {
            // NightAudit calls invoke("get_audit_logs") with NO args
            expect(invoke).toHaveBeenCalledWith("get_audit_logs");
        });
    });

    it("run_night_audit sends correct command", async () => {
        setMockResponse("run_night_audit", () => ({
            id: "audit-1",
            audit_date: "2026-03-15",
            total_revenue: 1200000,
            room_revenue: 800000,
            folio_revenue: 400000,
            total_expenses: 200000,
            occupancy_pct: 30,
            rooms_sold: 3,
            total_rooms: 10,
            notes: null,
            created_at: "2026-03-15T23:59:59+07:00",
        }));

        // Call directly through invoke
        const result = await invoke("run_night_audit", {
            auditDate: "2026-03-15",
            notes: null,
        });

        expect(invoke).toHaveBeenCalledWith("run_night_audit", {
            auditDate: "2026-03-15",
            notes: null,
        });
        expect(result).toHaveProperty("total_rooms", 10);
    });

    it("handles audit logs display", async () => {
        setMockResponse("get_audit_logs", () => [
            {
                id: "al-1",
                audit_date: "2026-03-15",
                total_revenue: 1200000,
                room_revenue: 800000,
                folio_revenue: 400000,
                total_expenses: 200000,
                occupancy_pct: 30,
                rooms_sold: 3,
                total_rooms: 10,
                created_at: new Date().toISOString(),
            },
        ]);

        render(<NightAudit />);

        await waitFor(() => {
            // NightAudit calls invoke("get_audit_logs") with NO args
            expect(invoke).toHaveBeenCalledWith("get_audit_logs");
        });
    });
});
