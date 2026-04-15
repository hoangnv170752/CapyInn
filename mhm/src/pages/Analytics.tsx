import { useState, useEffect } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import StatCard from "@/components/shared/StatCard";
import EmptyState from "@/components/shared/EmptyState";
import { TrendingUp, Users, BedDouble, DollarSign } from "lucide-react";
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell } from "recharts";
import { invoke } from "@tauri-apps/api/core";
import { fmtMoney } from "@/lib/format";
import type { AnalyticsData } from "@/types";

const ROOM_TYPE_COLORS = ["#3B82F6", "#93C5FD"];

const fmtShort = (n: number) => {
    if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
    if (n >= 1000) return (n / 1000).toFixed(0) + "K";
    return String(n);
};
const PERIOD_LABELS: Record<"7d" | "30d" | "90d", string> = {
    "7d": "7 ngày",
    "30d": "30 ngày",
    "90d": "90 ngày",
};

const EMPTY_ANALYTICS: AnalyticsData = {
    total_revenue: 0, occupancy_rate: 0, adr: 0, revpar: 0,
    daily_revenue: [], revenue_by_source: [], expenses_by_category: [], top_rooms: [],
};

export default function Analytics() {
    const [period, setPeriod] = useState<"7d" | "30d" | "90d">("7d");
    const [data, setData] = useState<AnalyticsData>(EMPTY_ANALYTICS);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        setLoading(true);
        invoke<AnalyticsData>("get_analytics", { period })
            .then((result) => {
                setData({ ...EMPTY_ANALYTICS, ...result });
                setLoading(false);
            })
            .catch((err) => {
                console.error("get_analytics error:", err);
                setData(EMPTY_ANALYTICS);
                setLoading(false);
            });
    }, [period]);

    if (loading) {
        return (
            <div className="flex items-center justify-center h-64 text-brand-muted">
                Đang tải dữ liệu...
            </div>
        );
    }

    const roomTypeData = [
        { name: "Occupied", value: Math.round(data.occupancy_rate), color: ROOM_TYPE_COLORS[0] },
        { name: "Vacant", value: Math.round(100 - data.occupancy_rate), color: ROOM_TYPE_COLORS[1] },
    ];

    return (
        <div className="flex flex-col gap-6">

            {/* Period Toggle */}
            <div className="flex items-center justify-between">
                <h2 className="text-lg font-bold">Business Intelligence</h2>
                <div className="flex bg-white rounded-xl p-1 shadow-soft border border-slate-100">
                    {(["7d", "30d", "90d"] as const).map((p) => (
                        <Button
                            key={p}
                            variant={period === p ? "default" : "ghost"}
                            size="sm"
                            className={`rounded-lg text-xs font-semibold px-4 ${period === p ? "bg-brand-primary text-white shadow-sm" : "text-brand-muted"}`}
                            onClick={() => setPeriod(p)}
                        >
                            {PERIOD_LABELS[p]}
                        </Button>
                    ))}
                </div>
            </div>

            {/* KPI Cards */}
            <div className="grid grid-cols-4 gap-4">
                <StatCard icon={<DollarSign />} label="Doanh thu" value={fmtMoney(data.total_revenue)} change={data.total_revenue > 0 ? "+?" : undefined} color="blue" layout="vertical" />
                <StatCard icon={<BedDouble />} label="Tỷ lệ lấp đầy" value={`${Math.round(data.occupancy_rate)}%`} color="emerald" layout="vertical" />
                <StatCard icon={<TrendingUp />} label="ADR" value={fmtMoney(data.adr)} color="amber" layout="vertical" />
                <StatCard icon={<Users />} label="RevPAR" value={fmtMoney(data.revpar)} color="purple" layout="vertical" />
            </div>

            {/* Row 2: Revenue Chart + Occupancy Pie */}
            <div className="grid grid-cols-3 gap-4">
                <Card className="col-span-2 p-5">
                    <h3 className="font-bold mb-4">Doanh thu theo ngày</h3>
                    {(data.daily_revenue ?? []).length > 0 ? (
                        <ResponsiveContainer width="100%" height={240}>
                            <AreaChart data={data.daily_revenue}>
                                <defs>
                                    <linearGradient id="colorRev" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#3B82F6" stopOpacity={0.15} />
                                        <stop offset="95%" stopColor="#3B82F6" stopOpacity={0} />
                                    </linearGradient>
                                </defs>
                                <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
                                <XAxis dataKey="date" tick={{ fontSize: 11, fill: "#94A3B8" }} />
                                <YAxis tick={{ fontSize: 11, fill: "#94A3B8" }} tickFormatter={(v) => fmtShort(v)} />
                                <Tooltip formatter={(v) => [fmtMoney(Number(v)), "Doanh thu"]} />
                                <Area type="monotone" dataKey="revenue" stroke="#3B82F6" strokeWidth={2} fill="url(#colorRev)" />
                            </AreaChart>
                        </ResponsiveContainer>
                    ) : (
                        <EmptyState message="Chưa có dữ liệu doanh thu trong khoảng thời gian này" className="h-60" />
                    )}
                </Card>

                <Card className="p-5 flex flex-col">
                    <h3 className="font-bold mb-4">Tỷ lệ lấp đầy</h3>
                    <div className="flex-1 flex items-center justify-center">
                        <ResponsiveContainer width="100%" height={180}>
                            <PieChart>
                                <Pie data={roomTypeData} cx="50%" cy="50%" innerRadius={50} outerRadius={75} paddingAngle={4} dataKey="value">
                                    {roomTypeData.map((entry, i) => (
                                        <Cell key={i} fill={entry.color} />
                                    ))}
                                </Pie>
                                <Tooltip />
                            </PieChart>
                        </ResponsiveContainer>
                    </div>
                    <div className="flex justify-center gap-6 mt-2">
                        {roomTypeData.map((d) => (
                            <div key={d.name} className="flex items-center gap-2">
                                <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: d.color }} />
                                <span className="text-xs font-medium text-brand-muted">{d.name} ({d.value}%)</span>
                            </div>
                        ))}
                    </div>
                </Card>
            </div>

            {/* Row 3: Revenue by Source + Top Rooms */}
            <div className="grid grid-cols-2 gap-4">
                <Card className="p-5">
                    <h3 className="font-bold mb-4">Doanh thu theo nguồn</h3>
                    {(data.revenue_by_source ?? []).length > 0 ? (
                        data.revenue_by_source.map((s) => {
                            const maxVal = Math.max(...(data.revenue_by_source ?? []).map((d) => d.value));
                            const pct = maxVal > 0 ? (s.value / maxVal) * 100 : 0;
                            return (
                                <div key={s.name} className="mb-3">
                                    <div className="flex items-center justify-between mb-1">
                                        <span className="text-sm font-medium">{s.name}</span>
                                        <span className="text-sm font-bold">{fmtMoney(s.value)}</span>
                                    </div>
                                    <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                                        <div className="h-full bg-brand-primary rounded-full transition-all" style={{ width: `${pct}%` }} />
                                    </div>
                                </div>
                            );
                        })
                    ) : (
                        <EmptyState message="Chưa có dữ liệu" />
                    )}
                </Card>

                <Card className="p-5">
                    <h3 className="font-bold mb-4">Top 5 phòng doanh thu cao</h3>
                    {(data.top_rooms ?? []).length > 0 ? (
                        <ResponsiveContainer width="100%" height={200}>
                            <BarChart data={data.top_rooms} layout="vertical">
                                <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" horizontal={false} />
                                <XAxis type="number" tick={{ fontSize: 11, fill: "#94A3B8" }} tickFormatter={(v) => fmtShort(v)} />
                                <YAxis type="category" dataKey="room" tick={{ fontSize: 12, fontWeight: 600, fill: "#334155" }} width={40} />
                                <Tooltip formatter={(v) => [fmtMoney(Number(v)), "Doanh thu"]} />
                                <Bar dataKey="revenue" fill="#3B82F6" radius={[0, 6, 6, 0]} barSize={20} />
                            </BarChart>
                        </ResponsiveContainer>
                    ) : (
                        <EmptyState message="Chưa có dữ liệu" />
                    )}
                </Card>
            </div>

            {/* Row 4: Expense Breakdown */}
            <Card className="p-5">
                <h3 className="font-bold mb-4">Chi phí theo danh mục</h3>
                {(data.expenses_by_category ?? []).length > 0 ? (
                    <div className="grid grid-cols-5 gap-6">
                        {data.expenses_by_category.map((e) => (
                            <div key={e.category}>
                                <span className="text-sm font-medium">{e.category}</span>
                                <p className="text-lg font-bold mt-1">{fmtMoney(e.amount)}</p>
                            </div>
                        ))}
                    </div>
                ) : (
                    <EmptyState message="Chưa có chi phí nào trong khoảng thời gian này" />
                )}
            </Card>
        </div>
    );
}

