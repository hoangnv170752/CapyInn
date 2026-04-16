import Section from "@/components/shared/Section";
import InfoItem from "@/components/shared/InfoItem";
import { User } from "lucide-react";
import type { Guest } from "@/types";

interface RoomGuestsSectionProps {
    guests: Guest[];
    mode: "page" | "sheet";
}

export default function RoomGuestsSection({ guests, mode }: RoomGuestsSectionProps) {
    if (guests.length === 0) {
        return null;
    }

    const title = mode === "page" ? "Thông tin khách" : `Khách hàng (${guests.length})`;

    return (
        <Section icon={User} title={title}>
            <div className="space-y-2">
                {guests.map((guest) => (
                    <div key={guest.id} className="bg-white border border-slate-100 rounded-xl p-3">
                        <div className="font-semibold text-sm text-slate-900">{guest.full_name}</div>
                        {mode === "page" ? (
                            <div className="grid grid-cols-2 gap-x-4 gap-y-1 mt-2 text-[12px]">
                                <InfoItem label="CCCD" value={guest.doc_number} variant="inline" />
                                <InfoItem label="Ngày sinh" value={guest.dob || "—"} variant="inline" />
                                <InfoItem label="Giới tính" value={guest.gender || "—"} variant="inline" />
                                <InfoItem label="Quốc tịch" value={guest.nationality || "—"} variant="inline" />
                                <InfoItem label="Địa chỉ" value={guest.address || "—"} variant="inline" className="col-span-2" />
                            </div>
                        ) : (
                            <div className="flex items-center justify-between mt-1.5 text-xs text-brand-muted">
                                <span className="font-mono">{guest.doc_number}</span>
                                <span>{guest.nationality || "VN"}</span>
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </Section>
    );
}
