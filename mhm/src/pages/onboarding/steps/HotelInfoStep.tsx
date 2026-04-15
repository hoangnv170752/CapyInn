type HotelInfoValue = {
  name: string;
  address: string;
  phone: string;
  defaultCheckinTime: string;
  defaultCheckoutTime: string;
};

export default function HotelInfoStep({
  value,
  onChange,
}: {
  value: HotelInfoValue;
  onChange: (next: Partial<HotelInfoValue>) => void;
}) {
  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-bold text-brand-text">Thông tin khách sạn</h1>
        <p className="text-sm text-brand-muted">Nhập thông tin cơ bản để hiển thị trên app và hóa đơn.</p>
      </div>

      <label className="block space-y-2">
        <span className="text-sm font-medium">Tên khách sạn</span>
        <input
          aria-label="Tên khách sạn"
          value={value.name}
          onChange={(event) => onChange({ name: event.target.value })}
          className="w-full rounded-xl border border-slate-200 px-4 py-3"
        />
      </label>

      <label className="block space-y-2">
        <span className="text-sm font-medium">Địa chỉ</span>
        <input
          aria-label="Địa chỉ"
          value={value.address}
          onChange={(event) => onChange({ address: event.target.value })}
          className="w-full rounded-xl border border-slate-200 px-4 py-3"
        />
      </label>

      <label className="block space-y-2">
        <span className="text-sm font-medium">Số điện thoại</span>
        <input
          aria-label="Số điện thoại"
          value={value.phone}
          onChange={(event) => onChange({ phone: event.target.value })}
          className="w-full rounded-xl border border-slate-200 px-4 py-3"
        />
      </label>

      <div className="grid grid-cols-2 gap-4">
        <label className="block space-y-2">
          <span className="text-sm font-medium">Giờ check-in</span>
          <input
            value={value.defaultCheckinTime}
            onChange={(event) => onChange({ defaultCheckinTime: event.target.value })}
            className="w-full rounded-xl border border-slate-200 px-4 py-3"
          />
        </label>
        <label className="block space-y-2">
          <span className="text-sm font-medium">Giờ check-out</span>
          <input
            value={value.defaultCheckoutTime}
            onChange={(event) => onChange({ defaultCheckoutTime: event.target.value })}
            className="w-full rounded-xl border border-slate-200 px-4 py-3"
          />
        </label>
      </div>
    </div>
  );
}
