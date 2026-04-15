import { ReactNode } from 'react';
import { Home, Calendar } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import AppLogo from '@/components/AppLogo';

export default function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-screen w-screen bg-brand-bg font-sans text-brand-text overflow-hidden select-none">
      {/* select-none để chống bôi đen text bậy bạ, tạo cảm giác Native App */}
      
      {/* 1. SIDEBAR TRÁI: Rộng cố định 260px, Nền trắng */}
      <aside className="w-[260px] bg-white border-r border-slate-100 flex flex-col p-6 z-20 shrink-0">
        <div className="mb-10 flex justify-center">
          <AppLogo className="h-12 w-12" />
        </div>
        
        {/* Navigation Menu */}
        <nav className="flex flex-col gap-2">
           <Button variant="ghost" className="justify-start bo góc xl font-medium !text-brand-text/90" size="lg">
             <Home className="mr-2" size={20} /> Dashboard
           </Button>
           <Button variant="ghost" className="justify-start opacity-60 hover:opacity-100 font-medium" size="lg">
             <Calendar className="mr-2" size={20} /> Timeline
           </Button>
        </nav>
      </aside>

      {/* 2. MAIN CONTENT AREA */}
      <main className="flex-1 flex flex-col h-full relative min-w-0">
        
        {/* HEADER: Trong suốt mờ (Backdrop Blur), Khu vực kéo thả cửa sổ App */}
        {/* data-tauri-drag-region: Cực kỳ quan trọng để user dùng chuột nắm kéo app trên Mac */}
        <header className="h-[88px] flex items-center justify-between px-10 bg-brand-bg/80 backdrop-blur-md sticky top-0 z-10 data-tauri-drag-region shrink-0">
          <div className="pointer-events-none">
            <h1 className="text-2xl font-bold tracking-tight">Thống kê</h1>
            <p className="text-sm text-brand-muted">Hôm nay</p>
          </div>
          
          <div className="flex items-center gap-4 pointer-events-auto">
             <Badge className="bg-green-50 text-green-700 border-0 rounded-full py-1.5 px-3 uppercase tracking-wider text-[10px] font-bold">
               ● Scanner Ready
             </Badge>
             <Button className="rounded-xl bg-brand-primary shadow-soft hover:shadow-float transition-all px-6 py-5">
               + Khách mới (Manual)
             </Button>
          </div>
        </header>

        {/* 3. VÙNG CUỘN NỘI DUNG (Scrollable Area) */}
        <div className="flex-1 overflow-y-auto px-10 pb-10">
          {children}
        </div>
        
      </main>
    </div>
  );
}
