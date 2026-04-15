import type { ReactNode } from "react";

interface ModalProps {
    title: string;
    children: ReactNode;
}

export default function Modal({ title, children }: ModalProps) {
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div className="bg-white rounded-2xl p-5 w-full max-w-sm shadow-2xl animate-fade-up">
                <h3 className="text-base font-bold text-slate-900 mb-4">{title}</h3>
                {children}
            </div>
        </div>
    );
}
