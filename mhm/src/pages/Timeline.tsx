import { useState } from "react";
import { ChevronLeft, ChevronRight, Search, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

// Mock data
const ROOMS = [
  { id: "201", type: "Thường" },
  { id: "202", type: "Thường" },
  { id: "301", type: "VIP" },
  { id: "302", type: "Thường" },
  { id: "401", type: "Gia đình" },
  { id: "402", type: "VIP" },
];

const DAYS = [
  { day: "T2", date: "12" },
  { day: "T3", date: "13" },
  { day: "T4", date: "14", isToday: true },
  { day: "T5", date: "15" },
  { day: "T6", date: "16" },
  { day: "T7", date: "17" },
  { day: "CN", date: "18" },
  { day: "T2", date: "19" },
  { day: "T3", date: "20" },
  { day: "T4", date: "21" },
];

const BOOKINGS = [
  { id: 1, room: "201", guest: "Nguyễn Văn A", start: 0, length: 2, status: "paid", color: "bg-blue-100 text-blue-700" },
  { id: 2, room: "202", guest: "Trần Thị B", start: 2, length: 4, status: "unpaid", color: "bg-orange-100 text-orange-700" },
  { id: 3, room: "301", guest: "Lê Văn C", start: 1, length: 3, status: "paid", color: "bg-blue-100 text-blue-700" },
  { id: 4, room: "401", guest: "Dương D", start: 5, length: 3, status: "partPaid", color: "bg-purple-100 text-purple-700" },
];

export default function Timeline() {
  const [currentMonth] = useState("Tháng 06, 2024");

  return (
    <div className="flex flex-col h-full bg-white rounded-3xl shadow-soft overflow-hidden">
      
      {/* Toolbar */}
      <div className="flex items-center justify-between p-4 border-b border-slate-100 bg-white z-20">
        <div className="flex items-center gap-4">
          <div className="flex items-center bg-slate-50 rounded-xl p-1">
            <Button variant="ghost" size="icon" className="h-8 w-8 rounded-lg hover:bg-white hover:shadow-sm">
              <ChevronLeft size={16} />
            </Button>
            <span className="px-4 font-semibold text-sm w-32 text-center">{currentMonth}</span>
            <Button variant="ghost" size="icon" className="h-8 w-8 rounded-lg hover:bg-white hover:shadow-sm">
              <ChevronRight size={16} />
            </Button>
          </div>
          <Button variant="outline" className="h-10 rounded-xl px-4 border-slate-200 text-slate-600 font-medium hover:bg-slate-50">
            Hôm nay
          </Button>
        </div>
        
        <div className="flex items-center gap-3">
          <div className="relative w-64">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
            <Input 
              placeholder="Tìm khách hàng, số phòng..." 
              className="pl-9 bg-slate-50 border-transparent rounded-xl h-10"
            />
          </div>
          <Button className="h-10 rounded-xl px-4 bg-brand-primary text-white shadow-soft">
            <Plus size={16} className="mr-2" /> Thêm Booking
          </Button>
        </div>
      </div>

      {/* Timeline Grid Container */}
      <div className="flex-1 flex flex-col min-h-0 overflow-hidden relative">
        
        {/* Header Row (Days) */}
        <div className="flex border-b border-slate-100 bg-white sticky top-0 z-10 w-max min-w-full">
          {/* Room Column Header */}
          <div className="w-[120px] shrink-0 border-r border-slate-100 bg-white shadow-[2px_0_10px_rgba(0,0,0,0.02)] sticky left-0 z-20 flex items-center justify-center">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">Phòng</span>
          </div>
          
          {/* Days */}
          <div className="flex">
            {DAYS.map((d, i) => (
              <div 
                key={i} 
                className={`w-[100px] shrink-0 border-r border-slate-50 flex flex-col items-center justify-center py-2 ${d.isToday ? 'bg-blue-50/30' : ''}`}
              >
                <span className={`text-[10px] font-semibold uppercase ${d.isToday ? 'text-brand-primary' : 'text-slate-400'}`}>
                  {d.day}
                </span>
                <span className={`text-lg font-bold ${d.isToday ? 'text-brand-primary' : 'text-slate-700'}`}>
                  {d.date}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Timeline Body (Scrollable) */}
        <div className="flex-1 overflow-auto w-max min-w-full relative">
          <div className="flex flex-col">
            {ROOMS.map((room) => {
              const roomBookings = BOOKINGS.filter(b => b.room === room.id);
              
              return (
                <div key={room.id} className="flex group border-b border-slate-50 h-[72px]">
                  
                  {/* Room Column (Sticky Left) */}
                  <div className="w-[120px] shrink-0 border-r border-slate-100 bg-white shadow-[2px_0_10px_rgba(0,0,0,0.02)] sticky left-0 z-10 flex flex-col items-center justify-center p-2 group-hover:bg-slate-50/50 transition-colors">
                    <span className="font-bold text-slate-700">{room.id}</span>
                    <Badge variant="outline" className="mt-1 text-[10px] py-0 px-1.5 h-4 border-slate-200 text-slate-500 rounded-md">
                      {room.type}
                    </Badge>
                  </div>
                  
                  {/* Grid Cells & Bookings */}
                  <div className="flex relative w-max">
                    {/* Background Grid Cells */}
                    {DAYS.map((d, colIndex) => (
                      <div 
                        key={colIndex} 
                        className={`w-[100px] shrink-0 border-r border-slate-50 ${d.isToday ? 'bg-blue-50/10' : ''} group-hover:bg-slate-50/30 transition-colors`}
                      />
                    ))}
                    
                    {/* Absolute Positioned Bookings for this room */}
                    {roomBookings.map((booking) => {
                      const leftPos = booking.start * 100;
                      const width = booking.length * 100;
                      
                      return (
                        <div 
                          key={booking.id}
                          className="absolute top-1/2 -translate-y-1/2 px-1 z-10 cursor-pointer"
                          style={{ left: `${leftPos}px`, width: `${width}px` }}
                        >
                          <div className={`h-[44px] w-full ${booking.color} rounded-xl px-3 flex flex-col justify-center shadow-sm hover:shadow-md hover:-translate-y-0.5 transition-all border border-white/40`}>
                            <span className="font-semibold text-sm truncate">{booking.guest}</span>
                            <span className="text-[10px] opacity-80 uppercase tracking-widest font-bold mt-0.5">{booking.status}</span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                  
                </div>
              );
            })}
          </div>
        </div>
        
      </div>
    </div>
  );
}
