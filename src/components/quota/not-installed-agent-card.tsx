import { motion } from "motion/react";
import { Download } from "lucide-react";
import type { AgentType } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProviderLogo } from "@/components/providers/provider-logo";
import { agentTypeToProviderType } from "@/lib/constants";

interface NotInstalledAgentCardProps {
  agentType: AgentType;
  label: string;
  onInstall: () => void;
}

export function NotInstalledAgentCard({
  agentType,
  label,
  onInstall,
}: NotInstalledAgentCardProps) {
  const providerType = agentTypeToProviderType(agentType);

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
              <ProviderLogo type={providerType} />
              <div className="flex flex-col min-w-0">
                <span className="text-sm font-semibold truncate">{label}</span>
              </div>
            </div>
            <div
              className={cn(
                "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                "bg-muted text-muted-foreground",
              )}
            >
              <div className="h-1 w-1 rounded-full bg-muted-foreground" />
              Not Installed
            </div>
          </div>
        </CardHeader>

        <CardContent className="flex flex-1 flex-col">
          <Button
            size="sm"
            variant="outline"
            className="w-full gap-2"
            onClick={onInstall}
          >
            <Download className="h-3.5 w-3.5" />
            Install
          </Button>
        </CardContent>
      </Card>
    </motion.div>
  );
}
