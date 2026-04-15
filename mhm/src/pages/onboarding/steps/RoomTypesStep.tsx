import type { OnboardingRoomTypeDraft } from "../types";
import { createRoomTypeDraft } from "../useOnboardingDraft";

export default function RoomTypesStep({
  value,
  onChange,
}: {
  value: OnboardingRoomTypeDraft[];
  onChange: (next: OnboardingRoomTypeDraft[]) => void;
}) {
  function updateRoomType(tempId: string, patch: Partial<OnboardingRoomTypeDraft>) {
    onChange(value.map((roomType) => (
      roomType.tempId === tempId ? { ...roomType, ...patch } : roomType
    )));
  }

  function removeRoomType(tempId: string) {
    onChange(value.filter((roomType) => roomType.tempId !== tempId));
  }

  return (
    <div className="space-y-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-brand-text">Loại phòng</h1>
          <p className="text-sm text-brand-muted">Tạo loại phòng và giá mặc định trước khi sinh sơ đồ phòng.</p>
        </div>
        <button
          type="button"
          onClick={() => onChange([...value, createRoomTypeDraft()])}
          className="rounded-xl border border-slate-300 px-4 py-3 text-sm font-medium cursor-pointer"
        >
          Thêm loại phòng
        </button>
      </div>

      <div className="grid gap-4">
        {value.map((roomType, index) => (
          <div key={roomType.tempId} className="rounded-2xl border border-slate-200 p-4">
            <div className="mb-4 flex items-center justify-between gap-3">
              <p className="font-semibold text-brand-text">Loại phòng {index + 1}</p>
              {value.length > 1 && (
                <button
                  type="button"
                  onClick={() => removeRoomType(roomType.tempId)}
                  className="text-sm font-medium text-red-500 cursor-pointer"
                >
                  Xóa
                </button>
              )}
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="block space-y-2">
                <span className="text-sm font-medium">Tên loại phòng</span>
                <input
                  aria-label={`Tên loại phòng ${index + 1}`}
                  value={roomType.name}
                  onChange={(event) => updateRoomType(roomType.tempId, { name: event.target.value })}
                  className="w-full rounded-xl border border-slate-200 px-4 py-3"
                />
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium">Giá cơ bản</span>
                <input
                  aria-label={`Giá cơ bản ${index + 1}`}
                  type="number"
                  min="0"
                  value={roomType.basePrice}
                  onChange={(event) => updateRoomType(roomType.tempId, {
                    basePrice: event.target.value === "" ? 0 : Number(event.target.value),
                  })}
                  className="w-full rounded-xl border border-slate-200 px-4 py-3"
                />
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium">Số khách chuẩn</span>
                <input
                  aria-label={`Số khách chuẩn ${index + 1}`}
                  type="number"
                  min="1"
                  value={roomType.maxGuests}
                  onChange={(event) => updateRoomType(roomType.tempId, {
                    maxGuests: event.target.value === "" ? 0 : Number(event.target.value),
                  })}
                  className="w-full rounded-xl border border-slate-200 px-4 py-3"
                />
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium">Phụ thu người thêm</span>
                <input
                  aria-label={`Phụ thu người thêm ${index + 1}`}
                  type="number"
                  min="0"
                  value={roomType.extraPersonFee}
                  onChange={(event) => updateRoomType(roomType.tempId, {
                    extraPersonFee: event.target.value === "" ? 0 : Number(event.target.value),
                  })}
                  className="w-full rounded-xl border border-slate-200 px-4 py-3"
                />
              </label>
            </div>

            <div className="mt-4 grid gap-4 md:grid-cols-[auto,1fr] md:items-center">
              <label className="flex items-center gap-3 rounded-xl border border-slate-200 px-4 py-3">
                <input
                  type="checkbox"
                  checked={roomType.defaultHasBalcony}
                  onChange={(event) => updateRoomType(roomType.tempId, { defaultHasBalcony: event.target.checked })}
                />
                <span>Ban công mặc định</span>
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium">Ghi chú giường</span>
                <input
                  aria-label={`Ghi chú giường ${index + 1}`}
                  value={roomType.bedNote ?? ""}
                  onChange={(event) => updateRoomType(roomType.tempId, { bedNote: event.target.value })}
                  className="w-full rounded-xl border border-slate-200 px-4 py-3"
                />
              </label>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
