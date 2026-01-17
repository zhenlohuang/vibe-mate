import { motion } from "motion/react";
import { Settings2, LogIn } from "lucide-react";
import type { Provider } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProviderLogo } from "./provider-logo";

interface AgentCardProps {
  provider: Provider;
  onSetDefault: (id: string) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (id: string) => void;
  onLogin?: (id: string) => void;
}

interface QuotaInfo {
  currentSession: { used: number; total: number };
  currentWeek: { used: number; total: number };
}

// Mock quota data - this will be replaced with real data later
function getMockQuota(providerName: string): QuotaInfo {
  const quotas: Record<string, QuotaInfo> = {
    "Claude Code": {
      currentSession: { used: 1.2, total: 4 },
      currentWeek: { used: 12.5, total: 20 },
    },
    Codex: {
      currentSession: { used: 0.8, total: 3 },
      currentWeek: { used: 8.3, total: 15 },
    },
    "Gemini CLI": {
      currentSession: { used: 2.1, total: 5 },
      currentWeek: { used: 15.7, total: 25 },
    },
  };
  return (
    quotas[providerName] || {
      currentSession: { used: 0, total: 4 },
      currentWeek: { used: 0, total: 20 },
    }
  );
}

function getStatusConfig(status: Provider["status"], isDefault: boolean) {
  if (isDefault) {
    return {
      label: "ACTIVE",
      className: "bg-success/20 text-success",
      dotClassName: "bg-success",
    };
  }
  switch (status) {
    case "Connected":
      return {
        label: "STANDBY",
        className: "bg-warning/20 text-warning",
        dotClassName: "bg-warning",
      };
    case "Disconnected":
      return {
        label: "INACTIVE",
        className: "bg-muted text-muted-foreground",
        dotClassName: "bg-muted-foreground",
      };
    case "Error":
      return {
        label: "ERROR",
        className: "bg-destructive/20 text-destructive",
        dotClassName: "bg-destructive",
      };
    default:
      return {
        label: "INACTIVE",
        className: "bg-muted text-muted-foreground",
        dotClassName: "bg-muted-foreground",
      };
  }
}

export function AgentCard({ provider, onEdit, onLogin }: AgentCardProps) {
  const statusConfig = getStatusConfig(provider.status, provider.isDefault);
  const quota = getMockQuota(provider.name);
  const isLoggedIn = provider.authPath !== null && provider.authPath !== undefined;

  const sessionPercentage = (quota.currentSession.used / quota.currentSession.total) * 100;
  const weekPercentage = (quota.currentWeek.used / quota.currentWeek.total) * 100;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.2 }}
    >
      <Card
        className={cn(
          "provider-card relative overflow-hidden",
          provider.isDefault && "ring-1 ring-primary/50"
        )}
      >
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <ProviderLogo type={provider.type} />
              <span className="text-sm font-semibold truncate">{provider.name}</span>
            </div>
            <div
              className={cn(
                "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                statusConfig.className
              )}
            >
              <div className={cn("h-1 w-1 rounded-full", statusConfig.dotClassName)} />
              {statusConfig.label}
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-3">
          {!isLoggedIn ? (
            <div className="py-4">
              <Button
                size="sm"
                variant="outline"
                className="w-full"
                onClick={() => onLogin?.(provider.id)}
              >
                <LogIn className="h-3.5 w-3.5 mr-2" />
                Login
              </Button>
            </div>
          ) : (
            <>
              {/* Current Session */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-[10px]">
                  <span className="font-medium uppercase tracking-wider text-muted-foreground">
                    Current Session
                  </span>
                  <span className="font-mono text-foreground/80">
                    {quota.currentSession.used}M / {quota.currentSession.total}M
                  </span>
                </div>
                <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                  <div
                    className="h-full bg-primary transition-all"
                    style={{ width: `${sessionPercentage}%` }}
                  />
                </div>
                <div className="text-[9px] text-muted-foreground">
                  {sessionPercentage.toFixed(1)}% used
                </div>
              </div>

              {/* Current Week */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-[10px]">
                  <span className="font-medium uppercase tracking-wider text-muted-foreground">
                    Current Week
                  </span>
                  <span className="font-mono text-foreground/80">
                    {quota.currentWeek.used}M / {quota.currentWeek.total}M
                  </span>
                </div>
                <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                  <div
                    className="h-full bg-primary transition-all"
                    style={{ width: `${weekPercentage}%` }}
                  />
                </div>
                <div className="text-[9px] text-muted-foreground">
                  {weekPercentage.toFixed(1)}% used
                </div>
              </div>
            </>
          )}

          {/* Settings Button */}
          <div className="flex items-center justify-end pt-1">
            <button
              onClick={() => onEdit(provider)}
              className="rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
            >
              <Settings2 className="h-3.5 w-3.5" />
            </button>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
