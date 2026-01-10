import { motion } from "motion/react";
import { Bot, Terminal, Settings2, Download } from "lucide-react";
import type { CodingAgent } from "@/types";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";

interface AgentCardProps {
  agent: CodingAgent;
  onRefresh: () => void;
  onLogin: () => void;
}

function getAgentIcon(agentType: string) {
  if (agentType === "ClaudeCode") {
    return (
      <div className="flex h-7 w-7 items-center justify-center rounded-md bg-orange-500/10 shrink-0">
        <Bot className="h-3.5 w-3.5 text-orange-400" />
      </div>
    );
  }
  if (agentType === "GeminiCLI") {
    return (
      <div className="flex h-7 w-7 items-center justify-center rounded-md bg-blue-500/10 shrink-0">
        <Terminal className="h-3.5 w-3.5 text-blue-400" />
      </div>
    );
  }
  return (
    <div className="flex h-7 w-7 items-center justify-center rounded-md bg-muted shrink-0">
      <Bot className="h-3.5 w-3.5 text-muted-foreground" />
    </div>
  );
}

export function AgentCard({
  agent,
  onRefresh,
  onLogin,
}: AgentCardProps) {
  const isInstalled = agent.status === "Installed" || agent.status === "Authenticated";

  // Mock data for context usage
  const contextUsed = agent.agentType === "ClaudeCode" ? 124 : 8;
  const contextLimit = agent.agentType === "ClaudeCode" ? 200 : 1000;
  const contextPercentage = (contextUsed / contextLimit) * 100;
  const lastPing = agent.agentType === "ClaudeCode" ? "24ms" : "5m";
  const agentId = agent.agentType === "ClaudeCode" ? "ag_82x9..." : agent.agentType === "GeminiCLI" ? "gm_44q2..." : "ag_legacy";

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
    >
      <Card className="overflow-hidden">
        <CardContent className="p-3">
          <div className="flex items-center gap-3">
            {/* Agent Icon */}
            {getAgentIcon(agent.agentType)}

            {/* Agent Info */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold">{agent.name}</span>
              </div>
              <div className="text-[10px] text-muted-foreground font-mono">
                ID: {agentId}
              </div>
            </div>

            {/* Context Usage */}
            {isInstalled && (
              <div className="flex items-center gap-3">
                <div className="text-right">
                  <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
                    Context
                  </div>
                  <div className="text-[11px] font-mono">
                    <span className={contextPercentage > 80 ? "text-warning" : "text-accent"}>
                      {contextUsed}k
                    </span>
                    <span className="text-muted-foreground">/{contextLimit >= 1000 ? "1M" : `${contextLimit}k`}</span>
                  </div>
                </div>
                <div className="w-20">
                  <Progress 
                    value={contextPercentage} 
                    className="h-1"
                  />
                </div>
              </div>
            )}

            {/* Last Ping */}
            <div className="text-right w-14">
              <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground">
                Ping
              </div>
              <div className="text-[11px] text-muted-foreground">
                {isInstalled ? lastPing : "--"}
              </div>
            </div>

            {/* Action Button */}
            <div className="w-24">
              {!isInstalled ? (
                <Button variant="outline" size="sm" className="w-full" onClick={onLogin}>
                  <Download className="mr-1 h-3 w-3" />
                  Install
                </Button>
              ) : (
                <Button variant="outline" size="sm" className="w-full" onClick={onRefresh}>
                  <Settings2 className="mr-1 h-3 w-3" />
                  Config
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
