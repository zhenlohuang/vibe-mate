import { motion } from "motion/react";
import { RefreshCw, Settings2 } from "lucide-react";
import type { Provider } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ProviderLogo } from "./provider-logo";

interface ProviderCardProps {
  provider: Provider;
  onEdit: (provider: Provider) => void;
  onTestConnection?: (id: string) => void;
}

function getStatusConfig(status: Provider["status"]) {
  switch (status) {
    case "Connected":
      return {
        label: "ACTIVE",
        className: "bg-success/20 text-success",
        dotClassName: "bg-success",
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
  onTestConnection,
}: ProviderCardProps) {
  const statusConfig = getStatusConfig(provider.status);
  const canTestConnection = typeof onTestConnection === "function";

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.2 }}
      className="h-full"
    >
      <Card className={cn("provider-card relative flex h-full flex-col overflow-hidden")}>
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <ProviderLogo type={provider.type} />
              <span className="text-sm font-semibold truncate">{provider.name}</span>
            </div>
            <div className="flex items-center gap-2">
              <div
                className={cn(
                  "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                  statusConfig.className
                )}
              >
                <div className={cn("h-1 w-1 rounded-full", statusConfig.dotClassName)} />
                {statusConfig.label}
              </div>
              {canTestConnection ? (
                <button
                  type="button"
                  onClick={() => onTestConnection(provider.id)}
                  className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                  aria-label={`Refresh ${provider.name}`}
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                </button>
              ) : null}
            </div>
          </div>
        </CardHeader>

        <CardContent className="flex flex-1 flex-col space-y-3">
          {/* Endpoint */}
          <div className="space-y-1">
            <div className="flex items-center justify-between text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
              <span>Endpoint</span>
            </div>
            <div className="rounded-md bg-secondary/50 px-2 py-1.5 font-mono text-[11px] text-foreground/80 truncate">
              {provider.apiBaseUrl || "Not configured"}
            </div>
          </div>

          {/* API Key */}
          <div className="space-y-1">
<div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
            Key
            </div>
            <div className="rounded-md bg-secondary/50 px-2 py-1.5 font-mono text-[11px] text-muted-foreground truncate min-w-0">
              {provider.apiKey ? `${provider.apiKey.slice(0, 7)}${"•".repeat(12)}` : "••••••••"}
            </div>
          </div>

          {/* Settings Button */}
          <div className="mt-auto flex items-center justify-end pt-1">
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
