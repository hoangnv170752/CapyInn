import type { OnboardingGeneratedRoom } from "../types";

type RoomPlanValue = {
  floors: number;
  roomsPerFloor: number;
  namingScheme: "floor_letter" | "floor_number" | "custom";
  columnAssignments: string[];
};

export default function RoomLayoutStep({
  value,
  roomTypes,
  generated,
  error,
  onChange,
  onGenerate,
}: {
  value: RoomPlanValue;
  roomTypes: string[];
  generated: OnboardingGeneratedRoom[];
  error: string | null;
  onChange: (next: Partial<RoomPlanValue>) => void;
  onGenerate: () => void;
}) {
  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-bold text-brand-text">Sơ đồ phòng</h1>
        <p className="text-sm text-brand-muted">
          Nhập số tầng, số phòng mỗi tầng và gán loại phòng theo từng cột để generate nhanh.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <label className="block space-y-2">
          <span className="text-sm font-medium">Số tầng</span>
          <input
            aria-label="Số tầng"
            type="number"
            min="1"
            value={value.floors}
            onChange={(event) => onChange({ floors: event.target.value === "" ? 0 : Number(event.target.value) })}
            className="w-full rounded-xl border border-slate-200 px-4 py-3"
          />
        </label>

        <label className="block space-y-2">
          <span className="text-sm font-medium">Số phòng mỗi tầng</span>
          <input
            aria-label="Số phòng mỗi tầng"
            type="number"
            min="1"
            value={value.roomsPerFloor}
            onChange={(event) => onChange({ roomsPerFloor: event.target.value === "" ? 0 : Number(event.target.value) })}
            className="w-full rounded-xl border border-slate-200 px-4 py-3"
          />
        </label>
      </div>

      <label className="block space-y-2">
        <span className="text-sm font-medium">Kiểu đánh số phòng</span>
        <select
          aria-label="Kiểu đánh số phòng"
          value={value.namingScheme}
          onChange={(event) => onChange({ namingScheme: event.target.value as RoomPlanValue["namingScheme"] })}
          className="w-full rounded-xl border border-slate-200 px-4 py-3 bg-white"
        >
          <option value="floor_letter">1A, 1B, 2A, 2B</option>
          <option value="floor_number">101, 102, 201, 202</option>
        </select>
      </label>

      <div className="grid gap-3">
        {Array.from({ length: value.roomsPerFloor }, (_, index) => (
          <label key={`column-${index}`} className="block space-y-2">
            <span className="text-sm font-medium">Cột {index + 1}</span>
            <select
              aria-label={`Cột ${index + 1}`}
              value={value.columnAssignments[index] ?? ""}
              onChange={(event) => {
                const nextAssignments = [...value.columnAssignments];
                nextAssignments[index] = event.target.value;
                onChange({ columnAssignments: nextAssignments });
              }}
              className="w-full rounded-xl border border-slate-200 px-4 py-3 bg-white"
            >
              {roomTypes.length === 0 && <option value="">Chưa có loại phòng hợp lệ</option>}
              {roomTypes.map((roomType) => (
                <option key={`${roomType}-${index}`} value={roomType}>
                  {roomType}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>

      <button
        type="button"
        onClick={onGenerate}
        className="rounded-xl border border-slate-300 px-4 py-3 font-medium cursor-pointer"
      >
        Tạo sơ đồ phòng
      </button>

      {error && <p className="text-sm text-red-500">{error}</p>}

      {generated.length > 0 && (
        <div className="rounded-2xl border border-slate-200 overflow-hidden">
          <div className="grid grid-cols-4 gap-4 bg-slate-50 px-4 py-3 text-xs font-semibold uppercase tracking-wide text-brand-muted">
            <span>Mã phòng</span>
            <span>Tầng</span>
            <span>Loại</span>
            <span>Giá</span>
          </div>
          <div className="divide-y divide-slate-100">
            {generated.map((room) => (
              <div key={room.id} className="grid grid-cols-4 gap-4 px-4 py-3 text-sm">
                <span>{room.id}</span>
                <span>{room.floor}</span>
                <span>{room.roomTypeName}</span>
                <span>{room.basePrice.toLocaleString("vi-VN")}đ</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
