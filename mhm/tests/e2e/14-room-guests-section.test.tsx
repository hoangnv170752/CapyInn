import { describe, expect, it } from "vitest";
import { render, screen } from "../helpers/render-app";
import RoomGuestsSection from "@/components/shared/RoomGuestsSection";
import { createGuest } from "../helpers/mock-data";

const guests = [
    createGuest({
        id: "g1",
        full_name: "Nguyễn Văn A",
        doc_number: "012345678901",
        nationality: "Việt Nam",
        dob: "1990-01-01",
        gender: "Nam",
        address: "102 Hoang Hoa Tham",
    }),
];

describe("14 — Room Guests Section", () => {
    it("renders the detailed guest layout for page mode", () => {
        render(<RoomGuestsSection guests={guests} mode="page" />);

        expect(screen.getByText("Thông tin khách")).toBeInTheDocument();
        expect(screen.getByText("Nguyễn Văn A")).toBeInTheDocument();
        expect(screen.getByText("012345678901")).toBeInTheDocument();
        expect(screen.getByText("102 Hoang Hoa Tham")).toBeInTheDocument();
    });

    it("renders the compact guest layout for sheet mode", () => {
        render(<RoomGuestsSection guests={guests} mode="sheet" />);

        expect(screen.getByText("Khách hàng (1)")).toBeInTheDocument();
        expect(screen.getByText("Nguyễn Văn A")).toBeInTheDocument();
        expect(screen.getByText("012345678901")).toBeInTheDocument();
        expect(screen.getByText("Việt Nam")).toBeInTheDocument();
    });
});
