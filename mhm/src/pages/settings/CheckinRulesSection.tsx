import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function CheckinRulesSection() {
  const [checkinTime, setCheckinTime] = useState("14:00");
  const [checkoutTime, setCheckoutTime] = useState("12:00");

  useEffect(() => {
    invoke<string | null>("get_settings", { key: "checkin_rules" })
      .then((value) => {
        if (!value) return;
        try {
          const data = JSON.parse(value);
          setCheckinTime(data.checkin || "14:00");
          setCheckoutTime(data.checkout || "12:00");
        } catch {
          // Ignore invalid saved settings and keep defaults.
        }
      })
      .catch(() => {});
  }, []);

  const handleSave = () => {
    const value = JSON.stringify({ checkin: checkinTime, checkout: checkoutTime });
    invoke("save_settings", { key: "checkin_rules", value })
      .then(() => toast.success("Đã lưu quy tắc check-in!"))
      .catch(() => toast.error("Lỗi khi lưu!"));
  };

  return (
    <div className="space-y-6 max-w-lg">
      <div>
        <h3 className="text-lg font-bold mb-1">Quy tắc Check-in</h3>
        <p className="text-sm text-brand-muted">Cấu hình giờ check-in/out mặc định</p>
      </div>

      <div className="space-y-4">
        <div>
          <Label>Giờ check-in mặc định</Label>
          <Input type="time" value={checkinTime} onChange={(event) => setCheckinTime(event.target.value)} className="mt-1.5 w-32" />
        </div>
        <div>
          <Label>Giờ check-out mặc định</Label>
          <Input type="time" value={checkoutTime} onChange={(event) => setCheckoutTime(event.target.value)} className="mt-1.5 w-32" />
        </div>
        <Button className="bg-brand-primary text-white rounded-xl" onClick={handleSave}>
          Lưu thay đổi
        </Button>
      </div>
    </div>
  );
}
