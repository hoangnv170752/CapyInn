import type { OnboardingGeneratedRoom } from "./types";

const LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export function generateRoomPlan(input: {
  floors: number;
  roomsPerFloor: number;
  namingScheme: "floor_letter" | "floor_number" | "custom";
  columnAssignments: string[];
  roomTypesByName: Record<string, {
    basePrice: number;
    maxGuests: number;
    extraPersonFee: number;
    defaultHasBalcony: boolean;
  }>;
  customFormatter?: (floor: number, columnIndex: number) => string;
}): { rooms: OnboardingGeneratedRoom[]; error: string | null } {
  if (input.floors < 1 || input.roomsPerFloor < 1) {
    return { rooms: [], error: "Số tầng và số phòng mỗi tầng phải lớn hơn 0" };
  }

  if (input.namingScheme === "floor_letter" && input.roomsPerFloor > LETTERS.length) {
    return { rooms: [], error: "Kiểu 1A, 1B chỉ hỗ trợ tối đa 26 phòng mỗi tầng" };
  }

  const rooms: OnboardingGeneratedRoom[] = [];
  const seen = new Set<string>();

  for (let floor = 1; floor <= input.floors; floor += 1) {
    for (let columnIndex = 0; columnIndex < input.roomsPerFloor; columnIndex += 1) {
      const roomTypeName = input.columnAssignments[columnIndex];
      if (!roomTypeName?.trim()) {
        return { rooms: [], error: `Chưa chọn loại phòng cho cột ${columnIndex + 1}` };
      }
      const roomType = input.roomTypesByName[roomTypeName];

      if (!roomType) {
        return { rooms: [], error: `Unknown room type: ${roomTypeName}` };
      }

      const id =
        input.namingScheme === "floor_letter" ? `${floor}${LETTERS[columnIndex]}` :
        input.namingScheme === "floor_number" ? `${floor}${String(columnIndex + 1).padStart(2, "0")}` :
        input.customFormatter?.(floor, columnIndex) ?? "";

      if (!id || seen.has(id)) {
        return { rooms: [], error: `Duplicate room id: ${id}` };
      }

      seen.add(id);
      rooms.push({
        id,
        name: `Phòng ${id}`,
        floor,
        roomTypeName,
        hasBalcony: roomType.defaultHasBalcony,
        basePrice: roomType.basePrice,
        maxGuests: roomType.maxGuests,
        extraPersonFee: roomType.extraPersonFee,
      });
    }
  }

  return { rooms, error: null };
}
