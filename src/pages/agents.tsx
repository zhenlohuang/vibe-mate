import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Loader2, RefreshCw } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentQuotaCard, NotInstalledAgentCard } from "@/components/quota";
import { useAgentAuth } from "@/hooks/use-agent-auth";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import { agentTypeToProviderType } from "@/lib/constants";
import { AGENT_TYPES, getAgentName } from "@/lib/agents";
import type { AgentType, AgentProviderType, AgentQuota } from "@/types";
import { Button } from "@/components/ui/button";
import { containerVariants, itemVariants } from "@/lib/animations";

export function AgentsPage() {
  const { accounts, isLoading: isAuthLoading, refetch, getQuota } = useAgentAuth();
  const { agents, isLoading: isAgentsLoading } = useAgents();
  const { toast } = useToast();
  const [hasRefreshedOnLoad, setHasRefreshedOnLoad] = useState(false);
  const [quotaByAgentType, setQuotaByAgentType] = useState<Record<string, AgentQuota | null>>({});
  const [quotaErrorByAgentType, setQuotaErrorByAgentType] = useState<Record<string, string | null>>({});
  const [isRefreshing, setIsRefreshing] = useState(false);

  const isLoading = isAuthLoading || isAgentsLoading;

  const discoveredAgentsMap = useMemo(() => {
    const map = new Map<AgentType, (typeof agents)[0]>();
    agents.forEach((a) => map.set(a.agentType, a));
    return map;
  }, [agents]);

  const accountByType = useMemo(() => {
    const map = new Map<AgentProviderType, (typeof accounts)[0]>();
    accounts.forEach((a) => map.set(a.agentType, a));
    return map;
  }, [accounts]);

  const loadQuotaForAgentType = useCallback(
    async (agentType: AgentProviderType) => {
      setQuotaErrorByAgentType((prev) => ({ ...prev, [agentType]: null }));
      try {
        const data = await getQuota(agentType);
        setQuotaByAgentType((prev) => ({ ...prev, [agentType]: data }));
      } catch (error) {
        setQuotaErrorByAgentType((prev) => ({ ...prev, [agentType]: String(error) }));
      }
    },
    [getQuota],
  );

  const refreshAllQuotas = useCallback(async () => {
    const installedProviderTypes = new Set<AgentProviderType>();
    agents.forEach((a) => {
      if (a.status !== "NotInstalled") {
        installedProviderTypes.add(agentTypeToProviderType(a.agentType));
      }
    });
    const refreshable = [...installedProviderTypes].filter(
      (t) => accountByType.get(t)?.isAuthenticated && t !== "GeminiCli",
    );
    await Promise.all(refreshable.map((t) => loadQuotaForAgentType(t)));
  }, [agents, accountByType, loadQuotaForAgentType]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refetch();
      await refreshAllQuotas();
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    if (isLoading || hasRefreshedOnLoad) return;
    setHasRefreshedOnLoad(true);
    void refreshAllQuotas();
  }, [hasRefreshedOnLoad, isLoading, refreshAllQuotas]);

  const handleInstall = useCallback(
    async (agentType: AgentType) => {
      const installUrls: Record<AgentType, string> = {
        ClaudeCode: "https://claude.ai/code",
        Codex: "https://github.com/codexyz/codex",
        GeminiCLI: "https://ai.google.dev/gemini-api/docs/cli",
        Antigravity: "https://antigravity.codes/download",
      };
      const url = installUrls[agentType];
      if (url) {
        window.open(url, "_blank");
        toast({
          title: "Opening Installation Page",
          description: `Opening the installation page for ${getAgentName(agentType)} in your browser.`,
        });
      } else {
        toast({
          title: "Error",
          description: "No installation URL for this agent.",
          variant: "destructive",
        });
      }
    },
    [toast],
  );

  if (isLoading) {
    return (
      <MainContent
        title="Coding Agents"
        description="Manage agents, view usage, and configure settings."
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
      description="Manage agents, view usage, and configure settings."
    >
      <div className="mb-6 flex items-center justify-end">
        <Button
          size="sm"
          variant="secondary"
          className="h-8 gap-2 px-3 text-[10px] uppercase tracking-wider"
          onClick={handleRefresh}
          disabled={isRefreshing}
        >
          {isRefreshing ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <RefreshCw className="h-3.5 w-3.5" />
          )}
          Refresh All
        </Button>
      </div>

      <motion.div
        variants={containerVariants}
        initial={false}
        animate="show"
        className="grid gap-4 grid-cols-1 md:grid-cols-2"
        style={{ gridAutoRows: "minmax(200px, auto)" }}
      >
        <AnimatePresence mode="popLayout">
          {AGENT_TYPES.map((agentType) => {
            const providerType = agentTypeToProviderType(agentType);
            const discovered = discoveredAgentsMap.get(agentType);
            const isInstalled = discovered != null && discovered.status !== "NotInstalled";
            const label = getAgentName(agentType);
            const account =
              accountByType.get(providerType) ?? ({
                agentType: providerType,
                isAuthenticated: false,
                email: null,
              });

            if (!isInstalled) {
              return (
                <motion.div key={agentType} variants={itemVariants} layout initial={false}>
                  <NotInstalledAgentCard
                    agentType={agentType}
                    label={label}
                    onInstall={() => handleInstall(agentType)}
                  />
                </motion.div>
              );
            }

            return (
              <motion.div key={agentType} variants={itemVariants} layout initial={false}>
                <AgentQuotaCard
                  account={account}
                  label={label}
                  quota={quotaByAgentType[providerType] ?? null}
                  quotaError={quotaErrorByAgentType[providerType] ?? null}
                  onRefresh={loadQuotaForAgentType}
                  configHref={`/agents/${agentType}/config`}
                  showConfigIcon
                />
              </motion.div>
            );
          })}
        </AnimatePresence>
      </motion.div>
    </MainContent>
  );
}
