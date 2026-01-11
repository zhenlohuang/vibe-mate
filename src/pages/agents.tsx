import { Loader2, Plus } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentCard } from "@/components/agents/agent-card";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import { useState } from "react";
import type { AgentType } from "@/types";
import { AGENT_TYPES, getDefaultConfigPath, getAgentName } from "@/lib/agents";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export function AgentsPage() {
  const { agents, isLoading, addAgent, removeAgent, updateAgentConfigPath } = useAgents();
  const { toast } = useToast();
  const [isAddMenuOpen, setIsAddMenuOpen] = useState(false);

  const enabledAgentTypes = new Set(agents.map((agent) => agent.agentType));
  const availableAgents = AGENT_TYPES.filter(
    (agentType) => !enabledAgentTypes.has(agentType)
  );

  const handleAddAgent = async (agentType: AgentType) => {
    try {
      await addAgent(agentType);
      setIsAddMenuOpen(false);
      toast({
        title: "Agent Added",
        description: `${getAgentName(agentType)} has been added to your workspace.`,
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleRemoveAgent = async (agentType: AgentType) => {
    try {
      await removeAgent(agentType);
      toast({
        title: "Agent Removed",
        description: `${getAgentName(agentType)} has been removed from your workspace.`,
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
        <div className="flex items-center justify-between">
          <div className="text-xs text-muted-foreground">
            Add the agents you want Vibe Mate to manage.
          </div>
          <DropdownMenu open={isAddMenuOpen} onOpenChange={setIsAddMenuOpen}>
            <DropdownMenuTrigger asChild>
              <Button size="sm" className="h-8 gap-2" disabled={availableAgents.length === 0}>
                <Plus className="h-3.5 w-3.5" />
                Add Agent
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {availableAgents.length === 0 ? (
                <DropdownMenuItem disabled>
                  All supported agents already added
                </DropdownMenuItem>
              ) : (
                availableAgents.map((agentType) => (
                  <DropdownMenuItem
                    key={agentType}
                    onClick={() => handleAddAgent(agentType)}
                  >
                    {getAgentName(agentType)}
                  </DropdownMenuItem>
                ))
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* Agent Cards */}
        <div className="space-y-3">
          {agents.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              No agents added yet. Use "Add Agent" to get started.
            </div>
          ) : (
            agents.map((agent) => (
              <AgentCard
                key={agent.agentType}
                agent={agent}
                defaultConfigPath={getDefaultConfigPath(agent.agentType)}
                onUpdateConfigPath={(configPath) =>
                  updateAgentConfigPath(agent.agentType, configPath)
                }
                onRemove={() => handleRemoveAgent(agent.agentType)}
              />
            ))
          )}
        </div>
      </div>
    </MainContent>
  );
}
