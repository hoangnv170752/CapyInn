import type { ComponentType, ReactNode } from "react";

import { cn } from "@/lib/utils";

interface SectionProps {
  icon: ComponentType<{ size?: number; className?: string }>;
  title: string;
  children: ReactNode;
  className?: string;
}

export default function Section({ icon: Icon, title, children, className }: SectionProps) {
  return (
    <div className={cn("bg-slate-50 border border-slate-100 rounded-xl p-4", className)}>
      <div className="flex items-center gap-2 mb-3">
        <Icon size={14} className="text-slate-400" />
        <h3 className="text-[11px] text-slate-400 font-semibold uppercase tracking-wider">{title}</h3>
      </div>
      {children}
    </div>
  );
}
