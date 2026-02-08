import { NavLink, useLocation } from "react-router-dom";
import { motion } from "motion/react";
import {
  Settings,
  Server,
  GitMerge,
  Bot,
  LayoutDashboard,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";

interface SidebarItem {
  id: string;
  label: string;
  icon: React.ElementType;
  href: string;
}

const menuItems: SidebarItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, href: "/" },
  { id: "agents", label: "Coding Agents", icon: Bot, href: "/agents" },
  {
    id: "providers",
    label: "Model Providers",
    icon: Server,
    href: "/providers",
  },
  { id: "router", label: "Routing Rules", icon: GitMerge, href: "/router" },
  {
    id: "preferences",
    label: "Preferences",
    icon: Settings,
    href: "/settings",
  },
];

export function Sidebar() {
  const location = useLocation();
  const proxyStatus = useAppStore((state) => state.proxyStatus);

  const renderMenuItem = (item: SidebarItem) => {
    const isActive =
      location.pathname === item.href ||
      (item.href !== "/" && location.pathname.startsWith(item.href));
    const Icon = item.icon;

    return (
      <NavLink key={item.id} to={item.href}>
        <motion.div
          initial={false}
          whileHover={{ x: 2 }}
          className={cn(
            "flex items-center gap-2.5 rounded-md px-2.5 py-2 text-sm font-medium transition-colors",
            isActive
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-secondary hover:text-foreground",
          )}
        >
          <Icon className="h-4 w-4 shrink-0" />
          <span className="truncate">{item.label}</span>
        </motion.div>
      </NavLink>
    );
  };

  return (
    <aside className="fixed left-0 top-0 z-40 h-screen w-[200px] border-r border-border bg-background flex flex-col">
      {/* Header with Logo */}
      <div className="flex h-14 items-center gap-3 px-3 border-b border-border">
        {/* Circular logo with gradient border */}
        <div className="relative flex h-8 w-8 items-center justify-center shrink-0">
          <div
            className="absolute inset-0 rounded-full p-[1.5px]"
            style={{
              background: "linear-gradient(135deg, #a855f7, #3b82f6, #22d3ee)",
            }}
          >
            <div className="flex h-full w-full items-center justify-center rounded-full bg-background">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                className="h-4 w-4"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline
                  points="4 17 10 11 4 5"
                  className="stroke-primary"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <line
                  x1="12"
                  y1="19"
                  x2="20"
                  y2="19"
                  className="stroke-primary"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </div>
          </div>
        </div>
        <span className="text-base font-semibold tracking-tight truncate">
          Vibe Mate
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 py-3 overflow-y-auto">
        <div className="space-y-0.5">{menuItems.map(renderMenuItem)}</div>
      </nav>

      {/* Status Indicator */}
      <div className="border-t border-border px-3 py-2.5">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-1.5">
            <span
              className={cn(
                "h-2 w-2 rounded-full shrink-0",
                proxyStatus.isRunning ? "bg-success" : "bg-error",
              )}
            />
            <span
              className={cn(
                "text-sm font-medium",
                proxyStatus.isRunning ? "text-success" : "text-error",
              )}
            >
              {proxyStatus.isRunning ? "Online" : "Offline"}
            </span>
          </div>
          <span className="text-meta font-mono text-muted-foreground">
            :{proxyStatus.port}
          </span>
        </div>
      </div>
    </aside>
  );
}
