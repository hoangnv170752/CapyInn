import { describe, expect, it } from "vitest";
import { generateRoomPlan } from "@/pages/onboarding/generateRoomPlan";

describe("13 — Onboarding Generator", () => {
  it("generates floor-letter room ids by column assignment", () => {
    const result = generateRoomPlan({
      floors: 2,
      roomsPerFloor: 2,
      namingScheme: "floor_letter",
      columnAssignments: ["Deluxe", "Standard"],
      roomTypesByName: {
        Deluxe: { basePrice: 500000, maxGuests: 4, extraPersonFee: 50000, defaultHasBalcony: true },
        Standard: { basePrice: 300000, maxGuests: 2, extraPersonFee: 100000, defaultHasBalcony: false },
      },
    });

    expect(result.rooms.map((room) => room.id)).toEqual(["1A", "1B", "2A", "2B"]);
    expect(result.rooms[0].hasBalcony).toBe(true);
    expect(result.rooms[1].roomTypeName).toBe("Standard");
  });

  it("returns an error when generated ids collide", () => {
    const result = generateRoomPlan({
      floors: 1,
      roomsPerFloor: 2,
      namingScheme: "custom",
      customFormatter: () => "101",
      columnAssignments: ["Standard", "Standard"],
      roomTypesByName: {
        Standard: { basePrice: 300000, maxGuests: 2, extraPersonFee: 0, defaultHasBalcony: false },
      },
    });

    expect(result.error).toMatch(/duplicate room id/i);
  });
});
