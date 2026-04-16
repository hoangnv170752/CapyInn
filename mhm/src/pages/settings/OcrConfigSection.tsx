import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { APP_RUNTIME_DIR } from "@/lib/appIdentity";

export default function OcrConfigSection() {
  return (
    <div className="space-y-6 max-w-lg">
      <div>
        <h3 className="text-lg font-bold mb-1">Cấu hình OCR</h3>
        <p className="text-sm text-brand-muted">Thiết lập quét CCCD tự động</p>
      </div>

      <div className="space-y-4">
        <div>
          <Label>Thư mục scan</Label>
          <Input defaultValue={`~/${APP_RUNTIME_DIR}/Scans`} className="mt-1.5" readOnly />
        </div>
        <div className="flex items-center justify-between p-4 bg-slate-50 rounded-xl">
          <div>
            <p className="font-medium text-sm">Tự động quét</p>
            <p className="text-xs text-brand-muted">Quét ngay khi phát hiện file mới</p>
          </div>
          <div className="w-11 h-6 bg-brand-primary rounded-full relative cursor-pointer">
            <div className="w-5 h-5 bg-white rounded-full absolute top-0.5 right-0.5 shadow-sm" />
          </div>
        </div>
      </div>
    </div>
  );
}
