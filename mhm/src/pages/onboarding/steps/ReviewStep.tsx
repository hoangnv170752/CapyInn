import type { OnboardingDraft, OnboardingGeneratedRoom } from "../types";

export default function ReviewStep({
  draft,
  generated,
  error,
  saving,
  onSubmit,
}: {
  draft: OnboardingDraft;
  generated: OnboardingGeneratedRoom[];
  error: string | null;
  saving: boolean;
  onSubmit: () => void;
}) {
  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-bold text-brand-text">Review</h1>
        <p className="text-sm text-brand-muted">Kiểm tra lại cấu hình trước khi ghi vào database.</p>
      </div>

      <div className="grid gap-3">
        <div className="rounded-2xl border border-slate-200 p-4">
          <p className="font-semibold text-brand-text">{draft.hotel.name}</p>
          <p className="text-sm text-brand-muted">{draft.hotel.address}</p>
          <p className="text-sm text-brand-muted">{draft.hotel.phone}</p>
        </div>
        <div className="rounded-2xl border border-slate-200 p-4 text-sm text-brand-muted">
          {draft.roomTypes.length} loại phòng, {generated.length} phòng được tạo
        </div>
        <div className="rounded-2xl border border-slate-200 p-4 text-sm text-brand-muted">
          App lock: {draft.appLock.enabled ? "PIN enabled" : "No PIN"}
        </div>
      </div>

      {error && <p className="text-sm text-red-500">{error}</p>}

      <button
        type="button"
        onClick={onSubmit}
        disabled={saving}
        className="rounded-xl bg-brand-primary px-5 py-3 text-white font-medium cursor-pointer disabled:opacity-60"
      >
        {saving ? "Đang lưu..." : "Hoàn tất thiết lập"}
      </button>
    </div>
  );
}
