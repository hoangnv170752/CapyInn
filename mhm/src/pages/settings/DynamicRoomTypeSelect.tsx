import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { RoomTypeItem } from "@/types";

interface DynamicRoomTypeSelectProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export default function DynamicRoomTypeSelect({
  value,
  onChange,
  disabled,
}: DynamicRoomTypeSelectProps) {
  const [roomTypes, setRoomTypes] = useState<RoomTypeItem[]>([]);

  useEffect(() => {
    invoke<RoomTypeItem[] | null>("get_room_types")
      .then((items) => setRoomTypes(Array.isArray(items) ? items : []))
      .catch(() => {});
  }, []);

  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      disabled={disabled}
      className="mt-1.5 w-full bg-white border border-slate-100 rounded-xl px-3 py-2.5 text-sm font-medium outline-none disabled:opacity-50"
    >
      {roomTypes.map((roomType) => (
        <option key={roomType.id} value={roomType.name}>
          {roomType.name}
        </option>
      ))}
    </select>
  );
}
