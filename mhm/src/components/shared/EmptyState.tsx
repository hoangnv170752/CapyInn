import type { ComponentType } from "react";

interface EmptyStateProps {
    icon?: ComponentType<{ size?: number; className?: string }>;
    message: string;
    className?: string;
}

export default function EmptyState({ icon: Icon, message, className = "" }: EmptyStateProps) {
    return (
        <div className={`flex flex-col items-center justify-center py-8 ${className}`}>
            {Icon && <Icon size={32} className="text-slate-300 mx-auto mb-3" />}
            <p className="text-sm text-brand-muted text-center">{message}</p>
        </div>
    );
}
