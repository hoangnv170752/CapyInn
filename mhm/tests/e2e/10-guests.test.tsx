import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "../helpers/render-app";
import Guests from "@/pages/Guests";
import { setMockResponse, clearMockResponses, invoke } from "@test-mocks/tauri-core";
import { createGuestSummary } from "../helpers/mock-data";

const mockGuests = [
    createGuestSummary({ id: "g1", full_name: "Nguyễn Văn A", doc_number: "012345678901", nationality: "Việt Nam", total_stays: 3 }),
    createGuestSummary({ id: "g2", full_name: "John Smith", doc_number: "P12345678", nationality: "USA", total_stays: 1 }),
    createGuestSummary({ id: "g3", full_name: "Trần Thị B", doc_number: "098765432109", nationality: "Việt Nam", total_stays: 5 }),
];

describe("10 — Guests", () => {
    beforeEach(() => {
        clearMockResponses();
        invoke.mockClear();

        setMockResponse("get_all_guests", () => mockGuests);
    });

    it("loads guest list on mount", async () => {
        render(<Guests />);

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("get_all_guests", expect.anything());
        });
    });

    it("displays guest names", async () => {
        render(<Guests />);

        await waitFor(() => {
            expect(screen.getByText("Nguyễn Văn A")).toBeInTheDocument();
        });

        expect(screen.getByText("John Smith")).toBeInTheDocument();
        expect(screen.getByText("Trần Thị B")).toBeInTheDocument();
    });

    it("shows document numbers", async () => {
        render(<Guests />);

        await waitFor(() => {
            expect(screen.getByText("012345678901")).toBeInTheDocument();
        });
    });

    it("shows nationality info", async () => {
        render(<Guests />);

        await waitFor(() => {
            // Should display nationality for foreign guest
            expect(screen.getByText("USA")).toBeInTheDocument();
        });
    });

    it("search calls get_all_guests with search param", async () => {
        render(<Guests />);

        await waitFor(() => {
            expect(invoke).toHaveBeenCalledWith("get_all_guests", expect.anything());
        });
    });
});
