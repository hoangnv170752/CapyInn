import { describe, it, expect, beforeEach } from "vitest";
import { setMockResponse, clearMockResponses, invoke } from "@test-mocks/tauri-core";
import { useHotelStore } from "@/stores/useHotelStore";
import { createAllRooms, createStats } from "../helpers/mock-data";

const mockRooms = createAllRooms();
const mockStats = createStats();

describe("05 — Check-out Flow", () => {
    beforeEach(() => {
        clearMockResponses();
        invoke.mockClear();
        useHotelStore.setState({
            rooms: mockRooms,
            stats: mockStats,
            activeTab: "dashboard",
            roomDetail: null,
            housekeepingTasks: [],
            loading: false,
            isCheckinOpen: false,
        });

        setMockResponse("get_rooms", () => mockRooms);
        setMockResponse("get_dashboard_stats", () => mockStats);
    });

    it("check_out calls correct invoke command", async () => {
        setMockResponse("check_out", () => undefined);

        await useHotelStore.getState().checkOut("booking-1", 400000);

        expect(invoke).toHaveBeenCalledWith("check_out", {
            req: { booking_id: "booking-1", final_paid: 400000 },
        });
    });

    it("refreshes rooms and stats after checkout", async () => {
        setMockResponse("check_out", () => undefined);

        await useHotelStore.getState().checkOut("booking-1");

        // Should refresh data
        expect(invoke).toHaveBeenCalledWith("get_rooms");
        expect(invoke).toHaveBeenCalledWith("get_dashboard_stats");
    });

    it("navigates to dashboard after checkout", async () => {
        setMockResponse("check_out", () => undefined);

        useHotelStore.setState({ activeTab: "rooms" });
        await useHotelStore.getState().checkOut("booking-1");

        expect(useHotelStore.getState().activeTab).toBe("dashboard");
    });

    it("handles checkout error", async () => {
        setMockResponse("check_out", () => {
            throw new Error("Booking not found");
        });

        await expect(
            useHotelStore.getState().checkOut("nonexistent")
        ).rejects.toThrow("Booking not found");

        expect(useHotelStore.getState().loading).toBe(false);
    });

    it("checkout with no final_paid sends undefined", async () => {
        setMockResponse("check_out", () => undefined);

        await useHotelStore.getState().checkOut("booking-1");

        expect(invoke).toHaveBeenCalledWith("check_out", {
            req: { booking_id: "booking-1", final_paid: undefined },
        });
    });
});
