import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface InfoItemProps {
  label: string;
  value: ReactNode;
  className?: string;
  variant?: "inline" | "stacked" | "block";
}

export default function InfoItem({
  label,
  value,
  className,
  variant = "stacked",
}: InfoItemProps) {
  if (variant === "inline") {
    return (
      <div className={className}>
        <span className="text-slate-400">{label}: </span>
        <span className="text-slate-900">{value}</span>
      </div>
    );
  }

  if (variant === "block") {
    return (
      <div className={className}>
        <div className="text-[11px] text-slate-400 font-medium">{label}</div>
        <div className="text-slate-900 font-medium mt-0.5">{value}</div>
      </div>
    );
  }

  return (
    <div className={cn("space-y-0.5", className)}>
      <span className="text-[11px] text-brand-muted font-medium">{label}</span>
      <p className="font-semibold text-sm">{value}</p>
    </div>
  );
}

