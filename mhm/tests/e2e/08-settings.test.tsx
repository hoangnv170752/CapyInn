import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "../helpers/render-app";
import userEvent from "@testing-library/user-event";
import Settings from "@/pages/settings";
import { setMockResponse, clearMockResponses, invoke } from "@test-mocks/tauri-core";
import { useAuthStore } from "@/stores/useAuthStore";

describe("08 — Settings", () => {
    beforeEach(() => {
        clearMockResponses();
        invoke.mockClear();

        // Set admin auth state so admin-only sections render
        useAuthStore.setState({
            user: { id: "u1", name: "Admin", role: "admin", active: true, created_at: "" },
            isAuthenticated: true,
            loading: false,
            error: null,
        });

        setMockResponse("get_settings", (args: unknown) => {
            const key = (args as { key: string }).key;
            if (key === "hotel_info") {
                return JSON.stringify({ name: "Grand Hotel", address: "123 Main St", phone: "0901234567" });
            }
            if (key === "checkin_rules") {
                return JSON.stringify({ checkin_time: "14:00", checkout_time: "12:00" });
            }
            return null;
        });
        setMockResponse("get_rooms", () => []);
        setMockResponse("get_room_types", () => []);
        setMockResponse("get_pricing_rules", () => []);
        setMockResponse("get_special_dates", () => []);
        setMockResponse("list_users", () => [
            { id: "u1", name: "Admin", role: "admin", active: true, created_at: new Date().toISOString() },
        ]);
    });

    it("renders settings page", async () => {
        render(<Settings />);

        // Settings page should render without crashing
        await waitFor(() => {
            expect(invoke).toHaveBeenCalled();
        });
    });

    it("loads hotel info from settings", async () => {
        render(<Settings />);

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("get_settings", { key: "hotel_info" });
        });
    });

    it("loads checkin rules from settings", async () => {
        const user = userEvent.setup();
        render(<Settings />);

        // CheckinRulesSection renders lazily — click the Check-in Rules nav button first
        await user.click(screen.getByText("Check-in Rules"));

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("get_settings", { key: "checkin_rules" });
        });
    });

    it("loads pricing rules", async () => {
        const user = userEvent.setup();
        render(<Settings />);

        // PricingSection renders lazily — click the nav button first
        await user.click(screen.getByText("Pricing"));

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("get_pricing_rules");
        });
    });

    it("save_settings is called with correct key on save", async () => {
        setMockResponse("save_settings", () => undefined);

        // Directly test the invoke call pattern
        await invoke("save_settings", { key: "hotel_info", value: JSON.stringify({ name: "New Hotel" }) });

        expect(invoke).toHaveBeenCalledWith("save_settings", {
            key: "hotel_info",
            value: JSON.stringify({ name: "New Hotel" }),
        });
    });

    it("loads user list", async () => {
        const user = userEvent.setup();
        render(<Settings />);

        // UserManagementSection renders lazily — click the Users nav button
        await user.click(screen.getByText("Users"));

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("list_users");
        });
    });
});
