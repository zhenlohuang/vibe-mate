import { motion, AnimatePresence } from "motion/react";
import { Bot, Settings2, Download } from "lucide-react";
import { useState } from "react";
import type { CodingAgent } from "@/types";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ClaudeCodeConfig } from "./claude-code-config";
import { CodexConfig } from "./codex-config";
import { GeminiCLIConfig } from "./gemini-cli-config";

interface AgentCardProps {
  agent: CodingAgent;
  defaultConfigPath: string;
  isInstalled: boolean;
  onUpdateConfigPath: (configPath: string) => Promise<void>;
  onInstall?: () => void;
}

function getAgentIcon(_agentType: string) {
  return (
    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted shrink-0">
      <Bot className="h-5 w-5 text-muted-foreground" />
    </div>
  );
}

export function AgentCard({
  agent,
  defaultConfigPath,
  isInstalled,
  onUpdateConfigPath,
  onInstall,
}: AgentCardProps) {
  const [isConfigOpen, setIsConfigOpen] = useState(false);

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
                {!isInstalled && (
                  <span className="text-[9px] font-medium uppercase tracking-wider text-muted-foreground px-2 py-0.5 rounded-full bg-muted">
                    Not Installed
                  </span>
                )}
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex items-center gap-2">
              {!isInstalled ? (
                <Button
                  variant="default"
                  size="sm"
                  className="h-8 gap-2"
                  onClick={onInstall}
                >
                  <Download className="h-3.5 w-3.5" />
                  Install
                </Button>
              ) : (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7"
                  onClick={handleConfigToggle}
                >
                  <Settings2
                    className={`h-4 w-4 transition-transform ${isConfigOpen ? "rotate-90" : ""}`}
                  />
                </Button>
              )}
            </div>
          </div>
        </CardContent>

        {/* Expandable Config Section - Only show if installed */}
        <AnimatePresence>
          {isConfigOpen && isInstalled && (
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
