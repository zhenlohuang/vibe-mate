import { CheckCircle2, XCircle, AlertCircle, HelpCircle } from "lucide-react";
import type { AgentStatus as AgentStatusType } from "@/types";
import { cn } from "@/lib/utils";

interface AgentStatusProps {
  status: AgentStatusType;
}

export function AgentStatus({ status }: AgentStatusProps) {
  const statusConfig: Record<
    AgentStatusType,
    {
      icon: React.ElementType;
      label: string;
      description: string;
      color: string;
    }
  > = {
    Installed: {
      icon: AlertCircle,
      label: "Installed",
      description: "Agent is installed but not authenticated",
      color: "text-warning",
    },
    NotInstalled: {
      icon: XCircle,
      label: "Not Installed",
      description: "Agent is not installed on this system",
      color: "text-destructive",
    },
    Authenticated: {
      icon: CheckCircle2,
      label: "Authenticated",
      description: "Agent is ready to use",
      color: "text-success",
    },
    NotAuthenticated: {
      icon: HelpCircle,
      label: "Not Authenticated",
      description: "Login required to use this agent",
      color: "text-muted-foreground",
    },
  };

  const config = statusConfig[status];
  const Icon = config.icon;

  return (
    <div className="flex items-start gap-3 rounded-lg border border-border p-3">
      <Icon className={cn("h-5 w-5 mt-0.5", config.color)} />
      <div>
        <div className={cn("font-medium", config.color)}>{config.label}</div>
        <div className="text-xs text-muted-foreground">{config.description}</div>
      </div>
    </div>
  );
}

