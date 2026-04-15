export default function WelcomeStep({ onStart }: { onStart: () => void }) {
  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <p className="text-sm font-medium uppercase tracking-[0.2em] text-brand-muted">Setup</p>
        <h1 className="text-3xl font-bold tracking-tight text-brand-text">Thiết lập lần đầu</h1>
        <p className="text-sm text-brand-muted">
          Hoàn tất onboarding trước khi vào ứng dụng.
        </p>
      </div>
      <button
        type="button"
        onClick={onStart}
        className="rounded-xl bg-brand-primary px-5 py-3 text-white font-medium cursor-pointer"
      >
        Bắt đầu thiết lập
      </button>
    </div>
  );
}
