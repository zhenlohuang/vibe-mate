import { motion, AnimatePresence } from "motion/react";
import { Bot, Settings2, Trash2 } from "lucide-react";
import { useState } from "react";
import type { CodingAgent } from "@/types";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ClaudeCodeConfig } from "./claude-code-config";
import { CodexConfig } from "./codex-config";
import { GeminiCLIConfig } from "./gemini-cli-config";

interface AgentCardProps {
  agent: CodingAgent;
  defaultConfigPath: string;
  onUpdateConfigPath: (configPath: string) => Promise<void>;
  onRemove: () => void;
}

function getAgentIcon(_agentType: string) {
  return (
    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted shrink-0">
      <Bot className="h-5 w-5 text-muted-foreground" />
    </div>
  );
}

// Mock data for different agent types
function getAgentQuota(agentType: string) {
  if (agentType === "ClaudeCode") {
    return {
      session: { 
        used: 1.2, 
        limit: 4, 
        percentage: 30,
        resetTime: "Jan 18 at 2am" 
      },
      week: { 
        used: 12.5, 
        limit: 20, 
        percentage: 62.5,
        resetTime: "Sunday 00:00" 
      },
    };
  }
  if (agentType === "Codex") {
    return {
      session: { 
        used: 4.8, 
        limit: 5, 
        percentage: 96,
        resetTime: "Jan 11 at 3pm" 
      },
      week: { 
        used: 15, 
        limit: 50, 
        percentage: 30,
        resetTime: "Sunday 00:00" 
      },
    };
  }
  if (agentType === "GeminiCLI") {
    return {
      session: { 
        used: 2.5, 
        limit: 5, 
        percentage: 50,
        resetTime: "Jan 12 at 8am" 
      },
      week: { 
        used: 25, 
        limit: 50, 
        percentage: 50,
        resetTime: "Sunday 00:00" 
      },
    };
  }
  return null;
}

export function AgentCard({
  agent,
  defaultConfigPath,
  onUpdateConfigPath,
  onRemove,
}: AgentCardProps) {
  const [isConfigOpen, setIsConfigOpen] = useState(false);
  const [isRemoveOpen, setIsRemoveOpen] = useState(false);
  const quota = getAgentQuota(agent.agentType);

  const handleConfigToggle = () => {
    setIsConfigOpen(!isConfigOpen);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
    >
      <Card className="overflow-hidden">
        <CardContent className="p-4">
          <div className="flex items-center gap-4">
            {/* Agent Icon */}
            {getAgentIcon(agent.agentType)}

            {/* Agent Info */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold">{agent.name}</span>
              </div>
            </div>

            {/* Quota Display - 根据agent类型显示不同的额度信息 */}
            {quota && (
              <div className="flex items-center gap-12 flex-1">
                {/* Session/5h Limit */}
                <div className="flex flex-col gap-1 min-w-[200px]">
                  <div className="flex items-center justify-between">
                    <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground whitespace-nowrap">
                      {agent.agentType === "ClaudeCode" ? "Current Session" : "5h Limit"}
                    </div>
                    <div className="text-[11px] font-mono">
                      <span className="text-primary">
                        {quota.session.percentage}%
                      </span>
                    </div>
                  </div>
                  <div className="relative">
                    <div className="w-full relative h-1.5 overflow-hidden rounded-full bg-secondary">
                      <div 
                        className="h-full transition-all bg-primary"
                        style={{ width: `${quota.session.percentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground mt-1 text-right">
                      Resets {quota.session.resetTime}
                    </div>
                  </div>
                </div>

                {/* Weekly/Current Week Limit */}
                <div className="flex flex-col gap-1 min-w-[200px]">
                  <div className="flex items-center justify-between">
                    <div className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground whitespace-nowrap">
                      {agent.agentType === "ClaudeCode" ? "Current Week" : "Weekly Limit"}
                    </div>
                    <div className="text-[11px] font-mono">
                      <span className="text-primary">
                        {quota.week.percentage}%
                      </span>
                    </div>
                  </div>
                  <div className="relative">
                    <div className="w-full relative h-1.5 overflow-hidden rounded-full bg-secondary">
                      <div 
                        className="h-full transition-all bg-primary"
                        style={{ width: `${quota.week.percentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground mt-1 text-right">
                      Resets {quota.week.resetTime}
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Action Button */}
            <div className="flex items-center gap-1">
              <Dialog open={isRemoveOpen} onOpenChange={setIsRemoveOpen}>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground hover:text-destructive"
                  onClick={() => setIsRemoveOpen(true)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>
                      Remove {agent.name}?
                    </DialogTitle>
                  </DialogHeader>
                  <div className="text-sm text-muted-foreground">
                    This removes the agent from Vibe Mate. Your local config file is unchanged.
                  </div>
                  <DialogFooter>
                    <Button
                      variant="outline"
                      onClick={() => setIsRemoveOpen(false)}
                    >
                      Cancel
                    </Button>
                    <Button
                      variant="destructive"
                      onClick={() => {
                        setIsRemoveOpen(false);
                        onRemove();
                      }}
                    >
                      Remove
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
              <Button 
                variant="ghost"
                size="icon" 
                className="h-7 w-7" 
                onClick={handleConfigToggle}
              >
                <Settings2 className={`h-4 w-4 transition-transform ${isConfigOpen ? "rotate-90" : ""}`} />
              </Button>
            </div>
          </div>
        </CardContent>

        {/* Expandable Config Section */}
        <AnimatePresence>
          {isConfigOpen && (
            <>
              {agent.agentType === "ClaudeCode" && (
                <ClaudeCodeConfig
                  configPath={agent.configPath}
                  defaultConfigPath={defaultConfigPath}
                  onUpdateConfigPath={onUpdateConfigPath}
                />
              )}
              {agent.agentType === "Codex" && (
                <CodexConfig
                  configPath={agent.configPath}
                  defaultConfigPath={defaultConfigPath}
                  onUpdateConfigPath={onUpdateConfigPath}
                />
              )}
              {agent.agentType === "GeminiCLI" && (
                <GeminiCLIConfig
                  configPath={agent.configPath}
                  defaultConfigPath={defaultConfigPath}
                  onUpdateConfigPath={onUpdateConfigPath}
                />
              )}
            </>
          )}
        </AnimatePresence>
      </Card>
    </motion.div>
  );
}
