import { ReactNode } from "react";
import { Sidebar } from "./Sidebar";
import { TitleBar } from "./TitleBar";

interface MainLayoutProps {
  children: ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function MainLayout({ children, activeTab, onTabChange }: MainLayoutProps) {
  return (
    // Root Container - 使用 CSS 变量背景
    <div className="h-screen w-screen flex flex-col overflow-hidden bg-[hsl(var(--background-hsl))] text-[hsl(var(--foreground-hsl))] font-sans select-none">

      {/* 装饰性背景光晕 */}
      <div className="fixed top-0 left-0 w-full h-[500px] bg-gradient-to-b from-blue-500/5 to-transparent pointer-events-none z-0" />
      <div className="fixed -top-[200px] -right-[200px] w-[600px] h-[600px] bg-purple-500/10 blur-[100px] rounded-full pointer-events-none z-0" />
      <div className="fixed bottom-0 left-0 w-[500px] h-[500px] bg-blue-500/5 blur-[80px] rounded-full pointer-events-none z-0" />

      {/* Title Bar */}
      <div className="relative z-50">
        <TitleBar />
      </div>

      {/* App Content */}
      <div className="flex-1 flex overflow-hidden relative z-10 p-2 gap-2">
        {/* Floating Sidebar */}
        <div className="w-[72px] lg:w-[240px] flex-shrink-0 transition-all duration-300 ease-[cubic-bezier(0.25,0.1,0.25,1)]">
          <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
        </div>

        {/* Floating Main Content Area */}
        <main className="flex-1 flex flex-col overflow-hidden relative glass-panel rounded-2xl border-white/40 dark:border-white/5 shadow-xl ring-1 ring-black/5">
          <div className="absolute inset-0 overflow-auto scroll-smooth">
            <div className="p-6 h-full animate-enter">
              {children}
            </div>
          </div>
        </main>
      </div>

      {/* Footer - Integrated roughly, distinct from sidebar now */}
      <div className="relative z-10 px-4 pb-2">
        {/* Footer content can be moved to sidebar or kept minimal here if needed, 
             but for this design let's integrate status into the Sidebar or keep it hidden/minimal */}
      </div>
    </div>
  );
}
