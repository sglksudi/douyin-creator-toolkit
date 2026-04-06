import { cn } from "@/lib/utils";
import { APP_VERSION } from "@/lib/appMeta";
import {
  Bot,
  BookOpen,
  Download,
  Info,
  Link,
  ListTodo,
  Settings,
  Video,
} from "lucide-react";

interface NavItem {
  id: string;
  label: string;
  icon: React.ReactNode;
}

const navItems: NavItem[] = [
  { id: "local-video", label: "本地视频", icon: <Video className="w-[18px] h-[18px]" /> },
  { id: "douyin-link", label: "抖音链接", icon: <Link className="w-[18px] h-[18px]" /> },
  { id: "video-download", label: "视频下载", icon: <Download className="w-[18px] h-[18px]" /> },
  { id: "agent-studio", label: "Agent Studio", icon: <Bot className="w-[18px] h-[18px]" /> },
  { id: "knowledge-base", label: "知识库", icon: <BookOpen className="w-[18px] h-[18px]" /> },
  { id: "tasks", label: "任务队列", icon: <ListTodo className="w-[18px] h-[18px]" /> },
  { id: "about", label: "关于软件", icon: <Info className="w-[18px] h-[18px]" /> },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <aside className="h-full flex flex-col glass-panel rounded-2xl border-white/40 dark:border-white/5 overflow-hidden">
      <div className="h-16 flex items-center gap-3 px-4 relative">
        <img src="/logo.png" alt="Logo" className="w-8 h-8 rounded-xl shadow-lg" />
        <div className="flex flex-col hidden lg:flex">
          <span className="font-bold text-[13px] tracking-tight leading-none text-zinc-800 dark:text-zinc-100">
            抖音创作工具箱
          </span>
          <span className="text-[10px] tracking-wider text-zinc-400 dark:text-zinc-500 mt-1">
            链接、本地、Agent、AI 分析
          </span>
        </div>
      </div>

      <nav className="flex-1 px-3 py-4 space-y-1">
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={cn(
                "group w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-[13px] font-medium transition-all duration-300 relative overflow-hidden",
                isActive
                  ? "text-blue-600 dark:text-blue-400"
                  : "text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 hover:bg-zinc-100/50 dark:hover:bg-white/5"
              )}
            >
              {isActive && (
                <div className="absolute inset-0 bg-blue-500/10 dark:bg-blue-500/20 border border-blue-200/50 dark:border-blue-500/20 rounded-xl" />
              )}

              <span
                className={cn(
                  "relative z-10 transition-transform duration-300",
                  isActive ? "scale-110" : "group-hover:scale-110"
                )}
              >
                {item.icon}
              </span>
              <span className="relative z-10 hidden lg:block whitespace-nowrap">{item.label}</span>

              {isActive && (
                <div className="absolute right-3 w-1.5 h-1.5 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)] hidden lg:block" />
              )}
            </button>
          );
        })}
      </nav>

      <div className="p-3 border-t border-zinc-200/50 dark:border-white/5 bg-zinc-50/50 dark:bg-white/5 backdrop-blur-sm">
        <button
          onClick={() => onTabChange("settings")}
          className={cn(
            "w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all duration-200 group",
            activeTab === "settings"
              ? "bg-white dark:bg-zinc-800 shadow-sm"
              : "hover:bg-white/50 dark:hover:bg-white/5"
          )}
        >
          <div className="w-8 h-8 rounded-full bg-zinc-200 dark:bg-zinc-700 flex items-center justify-center text-zinc-500 group-hover:text-zinc-700 dark:group-hover:text-zinc-300 transition-colors">
            <Settings className="w-4 h-4" />
          </div>
          <div className="hidden lg:flex flex-col items-start ml-0.5">
            <span className="text-[12px] font-medium text-zinc-700 dark:text-zinc-200">系统设置</span>
            <span className="text-[10px] text-zinc-400">{APP_VERSION}</span>
          </div>
        </button>
      </div>
    </aside>
  );
}
