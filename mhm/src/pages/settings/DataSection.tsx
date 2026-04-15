import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";

export default function DataSection() {
  const [exporting, setExporting] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);

  const handleExport = async () => {
    setExporting(true);
    try {
      const path = await invoke<string>("export_csv");
      setExportPath(path);
      toast.success("Xuất CSV thành công!");
    } catch (error) {
      console.error(error);
      toast.error("Lỗi xuất CSV!");
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="space-y-6 max-w-lg">
      <div>
        <h3 className="text-lg font-bold mb-1">Dữ liệu & Sao lưu</h3>
        <p className="text-sm text-brand-muted">Quản lý dữ liệu ứng dụng</p>
      </div>

      <div className="space-y-4">
        <div className="flex items-center justify-between p-4 bg-slate-50 rounded-xl">
          <div>
            <p className="font-medium text-sm">Xuất dữ liệu CSV</p>
            <p className="text-xs text-brand-muted">Tải về toàn bộ booking và khách hàng</p>
            {exportPath && (
              <p className="text-xs text-emerald-600 mt-1 flex items-center gap-1">
                <FolderOpen size={12} /> {exportPath}
              </p>
            )}
          </div>
          <Button variant="outline" className="rounded-xl" onClick={() => void handleExport()} disabled={exporting}>
            {exporting ? "Đang xuất..." : "Export CSV"}
          </Button>
        </div>

        <div className="flex items-center justify-between p-4 bg-slate-50 rounded-xl">
          <div>
            <p className="font-medium text-sm">Sao lưu Database</p>
            <p className="text-xs text-brand-muted">Tạo bản sao lưu file SQLite</p>
          </div>
          <Button variant="outline" className="rounded-xl">
            Backup
          </Button>
        </div>

        <div className="flex items-center justify-between p-4 bg-red-50 rounded-xl">
          <div>
            <p className="font-medium text-sm text-red-700">Xóa toàn bộ dữ liệu</p>
            <p className="text-xs text-red-500">Hành động này không thể hoàn tác!</p>
          </div>
          <Button variant="outline" className="rounded-xl border-red-200 text-red-600 hover:bg-red-100">
            Reset
          </Button>
        </div>
      </div>
    </div>
  );
}
