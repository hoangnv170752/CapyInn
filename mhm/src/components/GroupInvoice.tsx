import { fmtMoney, fmtDateShort } from "@/lib/format";
import type { GroupInvoiceData } from "@/types";

interface Props {
    data: GroupInvoiceData;
}

export default function GroupInvoice({ data }: Props) {
    return (
        <div className="bg-white rounded-2xl p-8 max-w-[600px] mx-auto shadow-soft" id="group-invoice">
            {/* Hotel Header */}
            <div className="text-center mb-6">
                <h1 className="text-xl font-bold tracking-tight">{data.hotel_name}</h1>
                {data.hotel_address && <p className="text-sm text-brand-muted">{data.hotel_address}</p>}
                {data.hotel_phone && <p className="text-sm text-brand-muted">ĐT: {data.hotel_phone}</p>}
                <hr className="my-4 border-slate-200" />
                <h2 className="text-lg font-bold uppercase tracking-wider">HÓA ĐƠN ĐOÀN</h2>
            </div>

            {/* Group Info */}
            <div className="space-y-1 mb-6 text-sm">
                <div className="flex justify-between">
                    <span className="text-brand-muted">Tên đoàn:</span>
                    <span className="font-semibold">{data.group.group_name}</span>
                </div>
                <div className="flex justify-between">
                    <span className="text-brand-muted">Trưởng đoàn:</span>
                    <span>{data.group.organizer_name}</span>
                </div>
                {data.group.organizer_phone && (
                    <div className="flex justify-between">
                        <span className="text-brand-muted">SĐT:</span>
                        <span>{data.group.organizer_phone}</span>
                    </div>
                )}
                <div className="flex justify-between">
                    <span className="text-brand-muted">Ngày tạo:</span>
                    <span>{fmtDateShort(data.group.created_at)}</span>
                </div>
            </div>

            {/* Room breakdown */}
            <div className="mb-6">
                <h3 className="text-sm font-bold uppercase text-brand-muted mb-2">Chi tiết phòng</h3>
                <table className="w-full text-sm">
                    <thead>
                        <tr className="border-b border-slate-200">
                            <th className="text-left py-1.5 font-semibold">Phòng</th>
                            <th className="text-left py-1.5 font-semibold">Khách</th>
                            <th className="text-center py-1.5 font-semibold">Đêm</th>
                            <th className="text-right py-1.5 font-semibold">Giá/đêm</th>
                            <th className="text-right py-1.5 font-semibold">Thành tiền</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.rooms.map((room, i) => (
                            <tr key={i} className="border-b border-slate-50">
                                <td className="py-1.5">{room.room_name}</td>
                                <td className="py-1.5">{room.guest_name}</td>
                                <td className="py-1.5 text-center">{room.nights}</td>
                                <td className="py-1.5 text-right">{fmtMoney(room.price_per_night)}</td>
                                <td className="py-1.5 text-right font-medium">{fmtMoney(room.total)}</td>
                            </tr>
                        ))}
                    </tbody>
                    <tfoot>
                        <tr className="border-t border-slate-200">
                            <td colSpan={4} className="py-1.5 font-semibold text-right">Tổng phòng:</td>
                            <td className="py-1.5 text-right font-bold">{fmtMoney(data.subtotal_rooms)}</td>
                        </tr>
                    </tfoot>
                </table>
            </div>

            {/* Services */}
            {data.services.length > 0 && (
                <div className="mb-6">
                    <h3 className="text-sm font-bold uppercase text-brand-muted mb-2">Dịch vụ kèm</h3>
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b border-slate-200">
                                <th className="text-left py-1.5 font-semibold">Dịch vụ</th>
                                <th className="text-center py-1.5 font-semibold">SL</th>
                                <th className="text-right py-1.5 font-semibold">Đơn giá</th>
                                <th className="text-right py-1.5 font-semibold">Thành tiền</th>
                            </tr>
                        </thead>
                        <tbody>
                            {data.services.map((svc) => (
                                <tr key={svc.id} className="border-b border-slate-50">
                                    <td className="py-1.5">{svc.name}</td>
                                    <td className="py-1.5 text-center">{svc.quantity}</td>
                                    <td className="py-1.5 text-right">{fmtMoney(svc.unit_price)}</td>
                                    <td className="py-1.5 text-right font-medium">{fmtMoney(svc.total_price)}</td>
                                </tr>
                            ))}
                        </tbody>
                        <tfoot>
                            <tr className="border-t border-slate-200">
                                <td colSpan={3} className="py-1.5 font-semibold text-right">Tổng dịch vụ:</td>
                                <td className="py-1.5 text-right font-bold">{fmtMoney(data.subtotal_services)}</td>
                            </tr>
                        </tfoot>
                    </table>
                </div>
            )}

            {/* Grand Total */}
            <div className="border-t-2 border-brand-primary pt-4 space-y-1.5">
                <div className="flex justify-between text-base font-bold">
                    <span>TỔNG CỘNG</span>
                    <span className="text-brand-primary">{fmtMoney(data.grand_total)}</span>
                </div>
                <div className="flex justify-between text-sm">
                    <span className="text-brand-muted">Đã thanh toán</span>
                    <span className="text-emerald-600 font-semibold">{fmtMoney(data.paid_amount)}</span>
                </div>
                <div className="flex justify-between text-base font-bold">
                    <span>CÒN LẠI</span>
                    <span className={data.balance_due > 0 ? "text-red-500" : "text-emerald-600"}>
                        {fmtMoney(data.balance_due)}
                    </span>
                </div>
            </div>
        </div>
    );
}
