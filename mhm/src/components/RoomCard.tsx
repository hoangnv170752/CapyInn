import { User, BedDouble, CalendarClock } from "lucide-react";
import { ROOM_STATUS_CARD_BG, ROOM_STATUS_TEXT, ROOM_TYPE_LABELS, STATUS_DOT_COLORS, STATUS_LABELS } from "@/lib/constants";
import { fmtMoney } from "@/lib/format";
import type { Room } from "@/types";

interface RoomCardProps {
  room: Room;
  onClick: (id: string) => void;
  nextReservationDate?: string | null;
}

export default function RoomCard({ room, onClick, nextReservationDate }: RoomCardProps) {
  return (
    <button
      onClick={() => onClick(room.id)}
      className={`
        relative p-4 rounded-xl border border-transparent cursor-pointer text-left
        transition-all duration-200 ease-out
        hover:shadow-soft active:scale-[0.98]
        ${ROOM_STATUS_CARD_BG[room.status]}
      `}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <span className="text-base font-bold text-text-primary">{room.id}</span>
        <span className={`w-2 h-2 rounded-full ${STATUS_DOT_COLORS[room.status]} status-dot`} />
      </div>

      {/* Room type icon + label */}
      <div className="flex items-center gap-1.5 mb-2">
        <BedDouble size={13} className="text-text-muted" />
        <span className="text-[11px] text-text-secondary font-medium">
          {ROOM_TYPE_LABELS[room.type] ?? room.type}
        </span>
      </div>

      {/* Price */}
      <div className="text-[13px] font-semibold text-text-primary mb-2">
        {fmtMoney(room.base_price)}
      </div>

      {/* Status */}
      <div className={`text-[10px] font-semibold uppercase tracking-wider ${ROOM_STATUS_TEXT[room.status]}`}>
        {STATUS_LABELS[room.status]}
      </div>

      {/* Upcoming reservation badge */}
      {nextReservationDate && room.status === "vacant" && (
        <div className="mt-2 flex items-center gap-1 text-[10px] text-blue-600 bg-blue-50 px-2 py-0.5 rounded-lg w-fit">
          <CalendarClock size={10} />
          <span className="font-semibold">Đặt {nextReservationDate}</span>
        </div>
      )}

      {/* Occupied indicator */}
      {room.status === "occupied" && (
        <div className="absolute top-3 right-3">
          <User size={14} className="text-accent-red/60" />
        </div>
      )}
    </button>
  );
}
