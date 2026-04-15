import { isValidElement, type ComponentType, type ReactNode } from "react";

type StatCardLayout = "horizontal" | "vertical" | "centered";

interface StatCardProps {
    icon: ComponentType<{ size?: number; className?: string }> | ReactNode;
    label: string;
    value: string | number;
    sub?: string;
    change?: string;
    color?: string;
    bgColor?: string;
    layout?: StatCardLayout;
    className?: string;
}

const COLOR_MAP: Record<string, string> = {
    blue: "bg-blue-50 text-blue-600",
    emerald: "bg-emerald-50 text-emerald-600",
    amber: "bg-amber-50 text-amber-600",
    purple: "bg-purple-50 text-purple-600",
};

function resolveColors(color?: string, bgColor?: string) {
    if (bgColor && color) return { bg: bgColor, fg: color };
    if (color && COLOR_MAP[color]) {
        const [bg, fg] = COLOR_MAP[color].split(" ");
        return { bg, fg };
    }
    return { bg: "bg-slate-50", fg: "text-slate-600" };
}

function renderIcon(
    icon: ComponentType<{ size?: number; className?: string }> | ReactNode,
    fg: string,
    iconSize: number,
) {
    // Already a rendered JSX element (e.g. <DollarSign />)
    if (isValidElement(icon)) return icon;

    // A component reference: function or ForwardRef object (lucide-react)
    if (typeof icon === "function" || (typeof icon === "object" && icon !== null && "$$typeof" in icon)) {
        const Icon = icon as ComponentType<{ size?: number; className?: string }>;
        return <Icon size={iconSize} className={fg} />;
    }

    return icon;
}

export default function StatCard({
    icon,
    label,
    value,
    sub,
    change,
    color,
    bgColor,
    layout = "horizontal",
    className = "",
}: StatCardProps) {
    const { bg, fg } = resolveColors(color, bgColor);

    if (layout === "centered") {
        return (
            <div className={`text-center ${className}`}>
                <div className={`w-9 h-9 rounded-xl mx-auto flex items-center justify-center ${bg} ${fg}`}>
                    {renderIcon(icon, fg, 16)}
                </div>
                <p className="text-lg font-bold mt-2">{value}</p>
                <p className="text-[11px] text-brand-muted">{label}</p>
            </div>
        );
    }

    if (layout === "vertical") {
        return (
            <div className={`bg-white rounded-2xl shadow-soft border border-slate-100 p-5 ${className}`}>
                <div className="flex items-center justify-between mb-3">
                    <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${bg} ${fg}`}>
                        {renderIcon(icon, fg, 20)}
                    </div>
                    {change && (
                        <span className="text-xs font-bold text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded-full">
                            {change}
                        </span>
                    )}
                </div>
                <p className="text-2xl font-bold">{value}</p>
                <p className="text-xs text-brand-muted mt-1">{label}</p>
            </div>
        );
    }

    // horizontal (default)
    return (
        <div className={`bg-white rounded-2xl shadow-soft p-5 flex items-start gap-4 hover:shadow-float transition-all cursor-default ${className}`}>
            <div className={`w-12 h-12 rounded-xl ${bg} flex items-center justify-center shrink-0`}>
                {renderIcon(icon, fg, 24)}
            </div>
            <div className="min-w-0">
                <div className="text-sm text-brand-muted font-medium mb-1 truncate">{label}</div>
                <div className="flex items-baseline gap-1.5">
                    <span className={`text-2xl font-bold ${fg} tabular-nums leading-none tracking-tight`}>
                        {value}
                    </span>
                    {sub && <span className="text-xs text-brand-muted font-medium">{sub}</span>}
                </div>
            </div>
        </div>
    );
}
