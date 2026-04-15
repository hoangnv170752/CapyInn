import { useEffect, useState, type ComponentType } from "react";
import { useHotelStore } from "../stores/useHotelStore";
import { Button } from "@/components/ui/button";
import { Sparkles, Play, CheckCircle2, Clock, RefreshCw } from "lucide-react";
import RoomDrawer from "@/components/RoomDrawer";

export default function Housekeeping() {
  const { housekeepingTasks, fetchHousekeeping, updateHousekeeping } = useHotelStore();
  const [drawerRoomId, setDrawerRoomId] = useState<string | null>(null);

  useEffect(() => { fetchHousekeeping(); }, []);

  const statusConfig: Record<string, { label: string; icon: ComponentType<{ size?: number; className?: string }>; color: string; bg: string; dotColor: string }> = {
    needs_cleaning: { label: "Cần dọn", icon: Clock, color: "text-amber-600", bg: "bg-amber-50", dotColor: "bg-amber-400" },
    cleaning: { label: "Đang dọn", icon: RefreshCw, color: "text-blue-600", bg: "bg-blue-50", dotColor: "bg-blue-400" },
    clean: { label: "Sạch", icon: CheckCircle2, color: "text-emerald-600", bg: "bg-emerald-50", dotColor: "bg-emerald-400" },
  };

  const nextStatus: Record<string, string | null> = {
    needs_cleaning: "cleaning",
    cleaning: "clean",
    clean: null,
  };

  const fmtTime = (iso: string) => {
    try { return new Date(iso).toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit" }); } catch { return iso; }
  };

  const handleDrawerClose = () => {
    setDrawerRoomId(null);
    fetchHousekeeping();
  };

  return (
    <div className="space-y-5 animate-fade-up">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-brand-text">Housekeeping</h1>
          <p className="text-[13px] text-brand-muted mt-0.5">Quản lý dọn phòng</p>
        </div>
        <Button variant="outline" size="sm" onClick={fetchHousekeeping} className="rounded-xl gap-1.5 cursor-pointer">
          <RefreshCw size={13} /> Refresh
        </Button>
      </div>

      {housekeepingTasks.length === 0 ? (
        <div className="bg-white border border-slate-100 rounded-2xl p-12 text-center">
          <Sparkles size={32} className="text-slate-300 mx-auto mb-3" />
          <p className="text-[13px] text-brand-muted">Tất cả phòng đã sạch ✨</p>
        </div>
      ) : (
        <div className="bg-white border border-slate-100 rounded-2xl overflow-hidden">
          {/* Table Header */}
          <div className="grid grid-cols-[1fr_120px_160px_140px] gap-4 px-5 py-3 border-b border-slate-100 bg-slate-50/50">
            <span className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Phòng</span>
            <span className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Trạng thái</span>
            <span className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Thời gian</span>
            <span className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider text-right">Thao tác</span>
          </div>

          {/* Task Rows */}
          {housekeepingTasks.map((task, idx) => {
            const cfg = statusConfig[task.status] || statusConfig.needs_cleaning;
            const next = nextStatus[task.status];
            const isLast = idx === housekeepingTasks.length - 1;

            return (
              <div
                key={task.id}
                className={"grid grid-cols-[1fr_120px_160px_140px] gap-4 items-center px-5 py-3.5 hover:bg-slate-50 transition-colors" + (!isLast ? " border-b border-slate-50" : "")}
              >
                {/* Room — clickable to open drawer */}
                <button
                  className="flex items-center gap-3 cursor-pointer text-left hover:opacity-80 transition-opacity"
                  onClick={() => setDrawerRoomId(task.room_id)}
                >
                  <div className="w-9 h-9 rounded-lg bg-slate-100 flex items-center justify-center font-bold text-brand-primary text-sm">
                    {task.room_id}
                  </div>
                  <span className="text-[14px] font-semibold text-slate-900">Phòng {task.room_id}</span>
                </button>

                {/* Status Badge */}
                <div>
                  <span className={"inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold " + cfg.color + " " + cfg.bg}>
                    <cfg.icon size={12} className={cfg.color} />
                    {cfg.label}
                  </span>
                </div>

                {/* Time */}
                <span className="text-[12px] text-slate-500">
                  {fmtTime(task.triggered_at)}
                </span>

                {/* Action */}
                <div className="text-right">
                  {next ? (
                    <Button
                      size="sm"
                      onClick={() => updateHousekeeping(task.id, next)}
                      className={"rounded-lg text-[12px] font-semibold cursor-pointer " + (next === "cleaning"
                        ? "bg-blue-500 hover:bg-blue-600 text-white"
                        : "bg-emerald-500 hover:bg-emerald-600 text-white"
                      )}
                    >
                      {next === "cleaning" ? <Play size={12} className="mr-1" /> : <CheckCircle2 size={12} className="mr-1" />}
                      {next === "cleaning" ? "Bắt đầu dọn" : "Dọn xong"}
                    </Button>
                  ) : (
                    <span className="text-[12px] text-emerald-500 font-medium">✓ Hoàn tất</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Room Drawer */}
      <RoomDrawer open={!!drawerRoomId} onClose={handleDrawerClose} roomId={drawerRoomId} />
    </div>
  );
}
