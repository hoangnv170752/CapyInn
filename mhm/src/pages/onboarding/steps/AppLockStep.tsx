type AppLockValue = {
  enabled: boolean;
  adminName: string;
  pin: string;
  confirmPin: string;
};

export default function AppLockStep({
  value,
  onChange,
}: {
  value: AppLockValue;
  onChange: (next: Partial<AppLockValue>) => void;
}) {
  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-bold text-brand-text">Khóa ứng dụng</h1>
        <p className="text-sm text-brand-muted">Chọn dùng PIN 4 chữ số hoặc bỏ qua để vào app trực tiếp.</p>
      </div>

      <label className="flex items-center gap-3 rounded-xl border border-slate-200 px-4 py-3">
        <input
          type="radio"
          name="app-lock"
          checked={!value.enabled}
          onChange={() => onChange({ enabled: false })}
        />
        <span>Không dùng PIN</span>
      </label>

      <label className="flex items-center gap-3 rounded-xl border border-slate-200 px-4 py-3">
        <input
          type="radio"
          name="app-lock"
          checked={value.enabled}
          onChange={() => onChange({ enabled: true })}
        />
        <span>Dùng PIN 4 chữ số</span>
      </label>

      {value.enabled && (
        <div className="grid gap-4">
          <label className="block space-y-2">
            <span className="text-sm font-medium">Tên admin</span>
            <input
              aria-label="Tên admin"
              value={value.adminName}
              onChange={(event) => onChange({ adminName: event.target.value })}
              className="w-full rounded-xl border border-slate-200 px-4 py-3"
            />
          </label>
          <label className="block space-y-2">
            <span className="text-sm font-medium">PIN</span>
            <input
              aria-label="PIN"
              value={value.pin}
              onChange={(event) => onChange({ pin: event.target.value.replace(/\D/g, "").slice(0, 4) })}
              className="w-full rounded-xl border border-slate-200 px-4 py-3"
            />
          </label>
          <label className="block space-y-2">
            <span className="text-sm font-medium">Xác nhận PIN</span>
            <input
              aria-label="Xác nhận PIN"
              value={value.confirmPin}
              onChange={(event) => onChange({ confirmPin: event.target.value.replace(/\D/g, "").slice(0, 4) })}
              className="w-full rounded-xl border border-slate-200 px-4 py-3"
            />
          </label>
        </div>
      )}
    </div>
  );
}
