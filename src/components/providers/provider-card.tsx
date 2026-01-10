import { motion } from "motion/react";
import { Settings2 } from "lucide-react";
import type { Provider } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { ProviderLogo } from "./provider-logo";

interface ProviderCardProps {
  provider: Provider;
  onSetDefault: (id: string) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (id: string) => void;
  onToggleProxy: (id: string, enabled: boolean) => void;
  onTestConnection: (id: string) => void;
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

export function ProviderCard({
  provider,
  onEdit,
  onToggleProxy,
}: ProviderCardProps) {
  const statusConfig = getStatusConfig(provider.status, provider.isDefault);

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
          {/* Endpoint */}
          <div className="space-y-1">
            <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
              Endpoint
            </div>
            <div className="rounded-md bg-secondary/50 px-2 py-1.5 font-mono text-[11px] text-foreground/80 truncate">
              {provider.apiBaseUrl}
            </div>
          </div>

          {/* API Key */}
          <div className="space-y-1">
            <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
              Key
            </div>
            <div className="rounded-md bg-secondary/50 px-2 py-1.5 font-mono text-[11px] text-muted-foreground">
              {provider.apiKey ? `${provider.apiKey.slice(0, 7)}${"•".repeat(24)}` : "••••••••"}
            </div>
          </div>

          {/* Proxy Toggle & Settings */}
          <div className="flex items-center justify-between pt-1">
            <div className="flex items-center gap-2">
              <Switch
                checked={provider.enableProxy}
                onCheckedChange={(checked) => onToggleProxy(provider.id, checked)}
                className="data-[state=checked]:bg-accent scale-90"
              />
              <span className="text-xs text-muted-foreground">Proxy</span>
            </div>
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
