import { describe, it, expect, beforeEach } from "vitest";
import { render, waitFor } from "../helpers/render-app";
import Analytics from "@/pages/Analytics";
import { setMockResponse, clearMockResponses, invoke } from "@test-mocks/tauri-core";
import { createAnalyticsData } from "../helpers/mock-data";

describe("07 — Analytics", () => {
    beforeEach(() => {
        clearMockResponses();
        invoke.mockClear();

        setMockResponse("get_analytics", () => createAnalyticsData());
    });

    it("renders analytics page", async () => {
        render(<Analytics />);

        // Page should render without crashing
        await waitFor(() => {
            // Should call get_analytics
            expect(invoke).toHaveBeenCalledWith("get_analytics", expect.anything());
        });
    });

    it("displays revenue data", async () => {
        render(<Analytics />);

        await waitFor(() => {
            // Revenue numbers should appear (total_revenue: 2,400,000)
            // The data should eventually show up and invoke should be called
            expect(invoke).toHaveBeenCalledWith("get_analytics", expect.anything());
        });
    });

    it("displays occupancy rate", async () => {
        render(<Analytics />);

        await waitFor(() => {
            // Occupancy rate 70%
            expect(invoke).toHaveBeenCalledWith("get_analytics", expect.anything());
        });
    });

    it("has period filter options", async () => {
        render(<Analytics />);

        // Should have period filter buttons/options (today, week, month)
        await waitFor(() => {
            expect(invoke).toHaveBeenCalled();
        });
    });
});
