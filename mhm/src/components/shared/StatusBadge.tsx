import { Badge } from "@/components/ui/badge";
import { STATUS_COLORS, STATUS_LABELS } from "@/lib/constants";
import type { RoomStatus } from "@/types";

interface StatusBadgeProps {
  status: RoomStatus;
  variant?: "pill" | "badge";
}

export default function StatusBadge({ status, variant = "pill" }: StatusBadgeProps) {
  return (
    <Badge
      className={`border ${STATUS_COLORS[status]} ${
        variant === "pill"
          ? "rounded-full px-3 py-1 text-[11px] font-semibold"
          : "rounded-lg px-3 py-1 text-xs font-bold"
      }`}
    >
      {STATUS_LABELS[status]}
    </Badge>
  );
}
