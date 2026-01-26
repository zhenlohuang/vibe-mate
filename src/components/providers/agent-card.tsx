import { motion } from "motion/react";
import { Settings2 } from "lucide-react";
import type { Provider } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ProviderLogo } from "./provider-logo";

interface AgentCardProps {
  provider: Provider;
  onEdit: (provider: Provider) => void;
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

export function AgentCard({ provider, onEdit }: AgentCardProps) {
  const isLoggedIn = provider.status === "Connected";
  const statusConfig = getStatusConfig(provider.status);

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
            <div
              className={cn(
                "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                statusConfig.className,
              )}
            >
              <div className={cn("h-1 w-1 rounded-full", statusConfig.dotClassName)} />
              {statusConfig.label}
            </div>
          </div>
        </CardHeader>

        <CardContent className="flex flex-1 flex-col space-y-3">
          <div className="space-y-1">
            <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
              Auth
            </div>
            <div className="rounded-md bg-secondary/50 px-2 py-1.5 text-[11px] text-foreground/80 truncate">
              {isLoggedIn ? "Authenticated" : "Not connected"}
            </div>
          </div>

          <div className="mt-auto flex items-center justify-end pt-1">
            <button
              type="button"
              onClick={() => onEdit(provider)}
              className="rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
              aria-label={`Edit ${provider.name}`}
            >
              <Settings2 className="h-3.5 w-3.5" />
            </button>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
