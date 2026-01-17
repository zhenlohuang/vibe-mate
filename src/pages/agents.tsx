import { Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentCard } from "@/components/agents/agent-card";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import type { AgentType } from "@/types";
import { AGENT_TYPES, getDefaultConfigPath, getAgentName } from "@/lib/agents";
import { invoke } from "@tauri-apps/api/core";

export function AgentsPage() {
  const { agents, isLoading, refetch } = useAgents();
  const { toast } = useToast();

  // Create a map of discovered agents for quick lookup
  const discoveredAgentsMap = new Map(
    agents.map((agent) => [agent.agentType, agent]),
  );

  // Create a list of all agents (both installed and not installed)
  const allAgents = AGENT_TYPES.map((agentType) => {
    const discoveredAgent = discoveredAgentsMap.get(agentType);
    return (
      discoveredAgent || {
        agentType,
        name: getAgentName(agentType),
        configPath: null,
        status: "NotInstalled" as const,
      }
    );
  });

  const handleInstall = async (agentType: AgentType) => {
    try {
      // Open the agent's installation/download page
      const installUrls: Record<AgentType, string> = {
        ClaudeCode: "https://claude.ai/code",
        Codex: "https://github.com/codexyz/codex",
        GeminiCLI: "https://ai.google.dev/gemini-api/docs/cli",
      };

      const url = installUrls[agentType];
      if (url) {
        window.open(url, "_blank");
        toast({
          title: "Opening Installation Page",
          description: `Opening the installation page for ${getAgentName(agentType)} in your browser.`,
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleUpdateConfigPath = async (
    agentType: AgentType,
    configPath: string,
  ) => {
    try {
      // Save the config path update
      await invoke("save_agent_config", {
        agentType,
        configPath,
      });

      // Refetch agents to get updated data
      await refetch();

      toast({
        title: "Config Updated",
        description: `Configuration path updated for ${getAgentName(agentType)}.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return (
      <MainContent
        title="Coding Agents"
        description="Monitor active agent instances, manage token consumption, and review live process output."
      >
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
        </div>
      </MainContent>
    );
  }

  return (
    <MainContent
      title="Coding Agents"
      description="Monitor active agent instances, manage token consumption, and review live process output."
    >
      <div className="space-y-4">
        <div className="text-xs text-muted-foreground">
          All supported coding agents are listed below. Install the agents you
          want to use with Vibe Mate.
        </div>

        {/* Agent Cards */}
        <div className="space-y-3">
          {allAgents.map((agent) => {
            const isInstalled = agent.status !== "NotInstalled";
            return (
              <AgentCard
                key={agent.agentType}
                agent={agent}
                defaultConfigPath={getDefaultConfigPath(agent.agentType)}
                isInstalled={isInstalled}
                onUpdateConfigPath={(configPath) =>
                  handleUpdateConfigPath(agent.agentType, configPath)
                }
                onInstall={() => handleInstall(agent.agentType)}
              />
            );
          })}
        </div>
      </div>
    </MainContent>
  );
}
