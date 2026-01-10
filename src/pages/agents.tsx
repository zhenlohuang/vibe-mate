import { motion } from "motion/react";
import { Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentCard } from "@/components/agents/agent-card";
import { TerminalOutput } from "@/components/agents/terminal-output";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import type { AgentType } from "@/types";
import { containerVariants, itemVariants } from "@/lib/animations";

export function AgentsPage() {
  const { agents, isLoading, checkStatus, openLogin } = useAgents();
  const { toast } = useToast();

  const handleRefresh = async (agentType: AgentType) => {
    try {
      await checkStatus(agentType);
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleLogin = async (agentType: AgentType) => {
    try {
      await openLogin(agentType);
      toast({
        title: "Login Initiated",
        description: "Please complete the login process in your browser.",
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
        {/* Agent Cards */}
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="show"
          className="space-y-3"
        >
          {agents.map((agent) => (
            <motion.div key={agent.agentType} variants={itemVariants}>
              <AgentCard
                agent={agent}
                onRefresh={() => handleRefresh(agent.agentType)}
                onLogin={() => handleLogin(agent.agentType)}
              />
            </motion.div>
          ))}
        </motion.div>

        {/* Empty State */}
        {agents.length === 0 && (
          <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-10">
            <h3 className="text-sm font-semibold">No Agents Detected</h3>
            <p className="mt-1.5 max-w-md text-center text-xs text-muted-foreground">
              No coding agents were found on your system. Install Claude Code or
              Gemini CLI to see them here.
            </p>
          </div>
        )}

        {/* Terminal Output */}
        <TerminalOutput />
      </div>
    </MainContent>
  );
}
