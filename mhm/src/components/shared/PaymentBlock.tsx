interface PaymentBlockProps {
    label: string;
    value: string;
    color: string;
}

export default function PaymentBlock({ label, value, color }: PaymentBlockProps) {
    return (
        <div>
            <div className="text-[11px] text-slate-400 font-medium mb-1">{label}</div>
            <div className={`text-base font-bold ${color} tabular-nums`}>{value}</div>
        </div>
    );
}
