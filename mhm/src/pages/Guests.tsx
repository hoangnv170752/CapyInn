import { useState, useEffect } from "react";
import { Search, Users, Star, ArrowUpRight } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { invoke } from "@tauri-apps/api/core";
import GuestProfileSheet from "@/components/GuestProfileSheet";
import StatCard from "@/components/shared/StatCard";
import { fmtDateShort, fmtMoney } from "@/lib/format";
import type { GuestSummary } from "@/types";

export default function Guests() {
    const [search, setSearch] = useState("");
    const [guests, setGuests] = useState<GuestSummary[]>([]);
    const [debouncedSearch, setDebouncedSearch] = useState("");
    const [selectedGuestId, setSelectedGuestId] = useState<string | null>(null);

    // Debounce search
    useEffect(() => {
        const timer = setTimeout(() => setDebouncedSearch(search), 300);
        return () => clearTimeout(timer);
    }, [search]);

    useEffect(() => {
        const s = debouncedSearch.trim() || undefined;
        invoke<GuestSummary[]>("get_all_guests", { search: s ?? null })
            .then(setGuests)
            .catch(() => setGuests([]));
    }, [debouncedSearch]);

    const totalSpent = guests.reduce((sum, g) => sum + g.total_spent, 0);
    const vipCount = guests.filter(g => g.total_stays >= 5).length;

    return (
        <div className="flex flex-col gap-6">

            {/* Stats Row */}
            <div className="grid grid-cols-3 gap-4">
                <StatCard icon={Users} label="Tổng số khách" value={guests.length} color="blue" />
                <StatCard icon={Star} label="Khách VIP" value={vipCount} color="amber" />
                <StatCard icon={ArrowUpRight} label="Tổng doanh thu từ khách" value={fmtMoney(totalSpent)} color="emerald" />
            </div>

            {/* Search + Table */}
            <Card className="p-0 overflow-hidden">
                <div className="p-5 border-b border-slate-100 flex items-center justify-between">
                    <h2 className="font-bold text-lg">Danh sách khách hàng</h2>
                    <div className="relative w-72">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                        <Input
                            placeholder="Tìm tên hoặc số CCCD..."
                            className="pl-9 bg-slate-50 border-transparent rounded-xl h-10"
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                        />
                    </div>
                </div>

                <Table>
                    <TableHeader>
                        <TableRow className="bg-slate-50/50">
                            <TableHead className="font-bold text-xs uppercase tracking-wider">Họ tên</TableHead>
                            <TableHead className="font-bold text-xs uppercase tracking-wider">CCCD</TableHead>
                            <TableHead className="font-bold text-xs uppercase tracking-wider">Quốc tịch</TableHead>
                            <TableHead className="font-bold text-xs uppercase tracking-wider text-center">Lần ở</TableHead>
                            <TableHead className="font-bold text-xs uppercase tracking-wider text-right">Tổng chi tiêu</TableHead>
                            <TableHead className="font-bold text-xs uppercase tracking-wider text-right">Lần cuối</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {guests.map((g) => (
                            <TableRow key={g.id} className="cursor-pointer hover:bg-slate-50 transition-colors" onClick={() => setSelectedGuestId(g.id)}>
                                <TableCell className="font-semibold flex items-center gap-2">
                                    {g.full_name}
                                    {g.total_stays >= 5 && <Badge className="bg-amber-50 text-amber-700 border-amber-200 rounded-md text-[10px] px-1.5 py-0">VIP</Badge>}
                                    {g.total_stays >= 2 && g.total_stays < 5 && <Badge className="bg-blue-50 text-blue-600 border-blue-200 rounded-md text-[10px] px-1.5 py-0">Returning</Badge>}
                                </TableCell>
                                <TableCell className="text-brand-muted font-mono text-sm">{g.doc_number}</TableCell>
                                <TableCell>{g.nationality || "—"}</TableCell>
                                <TableCell className="text-center font-semibold">{g.total_stays}</TableCell>
                                <TableCell className="text-right font-semibold">{fmtMoney(g.total_spent)}</TableCell>
                                <TableCell className="text-right text-brand-muted">{g.last_visit ? fmtDateShort(g.last_visit) : "—"}</TableCell>
                            </TableRow>
                        ))}
                        {guests.length === 0 && (
                            <TableRow>
                                <TableCell colSpan={6} className="text-center text-brand-muted py-12">
                                    {search ? "Không tìm thấy khách hàng nào" : "Chưa có khách hàng — Hãy check-in khách qua nút \"+ Khách mới\""}
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </Card>

            {selectedGuestId && (
                <GuestProfileSheet guestId={selectedGuestId} onClose={() => setSelectedGuestId(null)} />
            )}
        </div>
    );
}
