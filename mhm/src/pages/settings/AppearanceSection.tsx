import { useState } from "react";
import { toast } from "sonner";

import { Label } from "@/components/ui/label";
import { getLocale, setLocale, type Locale } from "@/lib/i18n";

export default function AppearanceSection() {
  const [locale, setLang] = useState<Locale>(getLocale());

  const handleLangChange = (newLocale: Locale) => {
    setLang(newLocale);
    setLocale(newLocale);
    toast.success(newLocale === "vi" ? "Đã chuyển sang Tiếng Việt" : "Switched to English");
  };

  return (
    <div className="space-y-6 max-w-lg">
      <div>
        <h3 className="text-lg font-bold mb-1">Giao diện</h3>
        <p className="text-sm text-brand-muted">Tùy chỉnh giao diện ứng dụng</p>
      </div>

      <div className="space-y-4">
        <div>
          <Label>Ngôn ngữ</Label>
          <select
            value={locale}
            onChange={(event) => handleLangChange(event.target.value as Locale)}
            className="mt-1.5 w-full bg-slate-50 border border-slate-100 rounded-xl px-3 py-2.5 text-sm font-medium outline-none"
          >
            <option value="vi">Tiếng Việt</option>
            <option value="en">English</option>
          </select>
        </div>
      </div>
    </div>
  );
}
