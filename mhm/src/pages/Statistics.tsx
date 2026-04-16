import { useState, useEffect, type ComponentType, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from "recharts";
import { TrendingUp, TrendingDown, PieChart, Plus, Download, X, Wallet, Receipt } from "lucide-react";
import { fmtMoney } from "@/lib/format";
import { EXPORT_PREFIX } from "@/lib/appIdentity";
import type { Expense, RevenueStats } from "@/types";

type Period = "today" | "week" | "month";
const PERIOD_LABELS: Record<Period, string> = {
  today: "Hôm nay",
  week: "Tuần",
  month: "Tháng",
};

const CATEGORY_LABELS: Record<string, string> = {
  electricity: "Điện", water: "Nước", garbage: "Rác",
  internet: "Internet", elevator: "Thang máy", other: "Khác",
};

export default function Statistics() {
  const [period, setPeriod] = useState<Period>("month");
  const [revenue, setRevenue] = useState<RevenueStats | null>(null);
  const [expenses, setExpenses] = useState<Expense[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [expCat, setExpCat] = useState("electricity");
  const [expAmt, setExpAmt] = useState(0);
  const [expNote, setExpNote] = useState("");
  const [expDate, setExpDate] = useState(new Date().toISOString().split("T")[0]);

  const getRange = (): [string, string] => {
    const now = new Date();
    const today = now.toISOString().split("T")[0];
    if (period === "today") return [today + "T00:00:00", today + "T23:59:59"];
    if (period === "week") { const d = new Date(now); d.setDate(d.getDate() - 7); return [d.toISOString(), now.toISOString()]; }
    const d = new Date(now.getFullYear(), now.getMonth(), 1);
    return [d.toISOString(), now.toISOString()];
  };

  const fetchData = async () => {
    const [from, to] = getRange();
    const rev = await invoke<RevenueStats>("get_revenue_stats", { from, to });
    setRevenue(rev);
    const exps = await invoke<Expense[]>("get_expenses", { from: from.split("T")[0], to: to.split("T")[0] });
    setExpenses(exps);
  };

  useEffect(() => { fetchData(); }, [period]);

  const totalExp = expenses.reduce((s, e) => s + e.amount, 0);
  const profit = (revenue?.total_revenue || 0) - totalExp;
  const handleAddExpense = async () => {
    await invoke("create_expense", { req: { category: expCat, amount: expAmt, note: expNote || null, expense_date: expDate } });
    setShowForm(false); setExpAmt(0); setExpNote(""); fetchData();
  };

  const handleCSV = () => {
    if (!revenue) return;
    let csv = "Loại,Ngày,Số tiền,Ghi chú\n";
    revenue.daily_revenue.forEach((d) => csv += `Doanh thu,${d.date},${d.revenue},Tiền phòng\n`);
    expenses.forEach((e) => csv += `Chi phí,${e.expense_date},${e.amount},${CATEGORY_LABELS[e.category] || e.category}${e.note ? " - " + e.note : ""}\n`);
    csv += `\nTổng doanh thu,,${revenue.total_revenue}\nTổng chi phí,,${totalExp}\nLợi nhuận,,${profit}\n`;
    const blob = new Blob(["\uFEFF" + csv], { type: "text/csv;charset=utf-8" });
    const a = document.createElement("a"); a.href = URL.createObjectURL(blob);
    a.download = `${EXPORT_PREFIX}-BaoCao-${new Date().toISOString().split("T")[0]}.csv`; a.click();
  };

  return (
    <div className="space-y-5 animate-fade-up">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-text-primary">Thống kê</h1>
          <p className="text-[13px] text-text-muted mt-0.5">Doanh thu & chi phí</p>
        </div>
        <div className="flex bg-bg-secondary rounded-lg p-0.5 border border-border-primary">
          {(["today", "week", "month"] as Period[]).map((p) => (
            <button key={p} onClick={() => setPeriod(p)}
              className={`px-3.5 py-1.5 rounded-md text-[12px] font-medium transition-colors cursor-pointer ${
                period === p ? "bg-accent-blue text-white" : "text-text-secondary hover:text-text-primary"
              }`}
            >
              {PERIOD_LABELS[p]}
            </button>
          ))}
        </div>
      </div>

      {/* Summary */}
      {revenue && (
        <div className="grid grid-cols-4 gap-3">
          <SummaryCard icon={TrendingUp} label="Doanh thu" value={fmtMoney(revenue.total_revenue)} color="text-accent-green" bg="bg-accent-green-soft" />
          <SummaryCard icon={TrendingDown} label="Chi phí" value={fmtMoney(totalExp)} color="text-accent-red" bg="bg-accent-red-soft" />
          <SummaryCard icon={Wallet} label="Lợi nhuận" value={fmtMoney(profit)} color={profit >= 0 ? "text-accent-blue" : "text-accent-red"} bg={profit >= 0 ? "bg-accent-blue-soft" : "bg-accent-red-soft"} />
          <SummaryCard icon={PieChart} label="Công suất" value={`${revenue.occupancy_rate.toFixed(0)}%`} sub={`${revenue.rooms_sold}/10`} color="text-accent-amber" bg="bg-accent-amber-soft" />
        </div>
      )}

      {/* Chart */}
      {revenue && revenue.daily_revenue.length > 0 && (
        <div className="glass-card p-5">
          <h3 className="text-[12px] text-text-muted font-semibold uppercase tracking-wider mb-4">Doanh thu theo ngày</h3>
          <ResponsiveContainer width="100%" height={220}>
            <BarChart data={revenue.daily_revenue}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2a3352" vertical={false} />
              <XAxis dataKey="date" tick={{ fill: "#5c6480", fontSize: 11 }} axisLine={{ stroke: "#2a3352" }} tickLine={false} />
              <YAxis tick={{ fill: "#5c6480", fontSize: 11 }} axisLine={false} tickLine={false} tickFormatter={(v) => `${(v / 1e6).toFixed(1)}tr`} />
              <Tooltip
                contentStyle={{ background: "#1a2035", border: "1px solid #2a3352", borderRadius: 10, fontSize: 12 }}
                labelStyle={{ color: "#8b93ab" }}
                formatter={(value) => [fmtMoney(Number(value)), "Doanh thu"]}
              />
              <Bar dataKey="revenue" fill="#4f8df5" radius={[5, 5, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Expenses */}
      <div className="glass-card p-5">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Receipt size={15} className="text-text-muted" />
            <h3 className="text-[12px] text-text-muted font-semibold uppercase tracking-wider">Chi phí</h3>
          </div>
          <button onClick={() => setShowForm(true)} className="flex items-center gap-1 px-3 py-1.5 bg-bg-elevated border border-border-primary rounded-lg text-[12px] text-text-secondary hover:text-text-primary transition-colors cursor-pointer">
            <Plus size={13} /> Thêm
          </button>
        </div>
        {expenses.length === 0 ? (
          <p className="text-[12px] text-text-muted text-center py-6">Chưa có chi phí</p>
        ) : (
          <div className="space-y-1">
            {expenses.map((e) => (
              <div key={e.id} className="flex items-center justify-between py-2.5 px-2 rounded-lg hover:bg-bg-hover transition-colors">
                <div>
                  <span className="text-[13px] text-text-primary font-medium">{CATEGORY_LABELS[e.category] || e.category}</span>
                  {e.note && <span className="text-[11px] text-text-muted ml-2">— {e.note}</span>}
                </div>
                <div className="text-right">
                  <div className="text-[13px] text-accent-red font-semibold tabular-nums">{fmtMoney(e.amount)}</div>
                  <div className="text-[10px] text-text-muted">{e.expense_date}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Export */}
      <button onClick={handleCSV} className="w-full flex items-center justify-center gap-2 py-3 bg-accent-green hover:bg-accent-green/90 text-bg-primary rounded-xl font-semibold text-[13px] transition-colors cursor-pointer">
        <Download size={15} /> Export CSV
      </button>

      {/* Expense modal */}
      {showForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="glass-card p-5 w-full max-w-sm shadow-2xl shadow-black/30 animate-fade-up">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-base font-bold text-text-primary">Thêm chi phí</h3>
              <button onClick={() => setShowForm(false)} className="text-text-muted hover:text-text-primary cursor-pointer"><X size={18} /></button>
            </div>
            <div className="space-y-3">
              <Field label="Loại">
                <select value={expCat} onChange={(e) => setExpCat(e.target.value)} className="w-full bg-bg-elevated border border-border-primary rounded-lg px-3 py-2 text-text-primary text-[13px]">
                  {Object.entries(CATEGORY_LABELS).map(([k, v]) => <option key={k} value={k}>{v}</option>)}
                </select>
              </Field>
              <Field label="Số tiền"><input type="number" value={expAmt} onChange={(e) => setExpAmt(Number(e.target.value))} className="w-full bg-bg-elevated border border-border-primary rounded-lg px-3 py-2 text-text-primary text-[13px]" /></Field>
              <Field label="Ngày"><input type="date" value={expDate} onChange={(e) => setExpDate(e.target.value)} className="w-full bg-bg-elevated border border-border-primary rounded-lg px-3 py-2 text-text-primary text-[13px]" /></Field>
              <Field label="Ghi chú"><input type="text" value={expNote} onChange={(e) => setExpNote(e.target.value)} className="w-full bg-bg-elevated border border-border-primary rounded-lg px-3 py-2 text-text-primary text-[13px]" placeholder="Tùy chọn" /></Field>
            </div>
            <div className="flex gap-2.5 mt-5">
              <button onClick={() => setShowForm(false)} className="flex-1 py-2.5 bg-bg-elevated hover:bg-bg-hover text-text-secondary rounded-xl text-[13px] cursor-pointer transition-colors">Hủy</button>
              <button onClick={handleAddExpense} disabled={expAmt <= 0} className="flex-1 py-2.5 bg-accent-blue hover:bg-accent-blue/90 disabled:opacity-40 text-white rounded-xl text-[13px] font-semibold cursor-pointer transition-colors">Lưu</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function SummaryCard({ icon: Icon, label, value, sub, color, bg }: { icon: ComponentType<{ size?: number; className?: string }>; label: string; value: string; sub?: string; color: string; bg: string }) {
  return (
    <div className="glass-card-sm p-4 flex items-start gap-3">
      <div className={`w-9 h-9 rounded-lg ${bg} flex items-center justify-center shrink-0`}><Icon size={18} className={color} /></div>
      <div className="min-w-0">
        <div className="text-[11px] text-text-muted font-medium mb-1">{label}</div>
        <div className="flex items-baseline gap-1">
          <span className={`text-lg font-bold ${color} tabular-nums leading-none`}>{value}</span>
          {sub && <span className="text-[11px] text-text-muted">{sub}</span>}
        </div>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return <div><label className="text-[11px] text-text-muted font-medium block mb-1">{label}</label>{children}</div>;
}
