import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentQuotaCard, NotInstalledAgentCard } from "@/components/quota";
import { useAgentAuth } from "@/hooks/use-agent-auth";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import { agentTypeToProviderType } from "@/lib/constants";
import { getAgentName } from "@/lib/agents";
import type { AgentType, AgentProviderType, AgentQuota } from "@/types";
import { containerVariants, itemVariants } from "@/lib/animations";

export function AgentsPage() {
  const { accounts, isLoading: isAuthLoading, getQuota } = useAgentAuth();
  const { agents, isLoading: isAgentsLoading } = useAgents();
  const { toast } = useToast();
  const [hasRefreshedOnLoad, setHasRefreshedOnLoad] = useState(false);
  const [quotaByAgentType, setQuotaByAgentType] = useState<Record<string, AgentQuota | null>>({});
  const [quotaErrorByAgentType, setQuotaErrorByAgentType] = useState<Record<string, string | null>>({});
  const [refreshingAgentTypes, setRefreshingAgentTypes] = useState<Set<AgentProviderType>>(new Set());

  const isLoading = isAuthLoading || isAgentsLoading;

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

  const refreshableTypes = useMemo(() => {
    const installed = new Set<AgentProviderType>();
    agents.forEach((a) => {
      if (a.status !== "NotInstalled") {
        installed.add(agentTypeToProviderType(a.agentType));
      }
    });
    return [...installed].filter(
      (t) => accountByType.get(t)?.isAuthenticated && t !== "GeminiCli",
    );
  }, [agents, accountByType]);

  const refreshAllQuotas = useCallback(
    async (types?: AgentProviderType[]) => {
      const toRefresh = types ?? refreshableTypes;
      await Promise.all(toRefresh.map((t) => loadQuotaForAgentType(t)));
    },
    [refreshableTypes, loadQuotaForAgentType],
  );

  const runQuotaRefresh = useCallback(async () => {
    setRefreshingAgentTypes((prev) => new Set([...prev, ...refreshableTypes]));
    try {
      await refreshAllQuotas();
    } finally {
      setRefreshingAgentTypes((prev) => {
        const next = new Set(prev);
        refreshableTypes.forEach((t) => next.delete(t));
        return next;
      });
    }
  }, [refreshAllQuotas, refreshableTypes]);

  const handleCardRefresh = useCallback(
    async (agentType: AgentProviderType) => {
      setRefreshingAgentTypes((prev) => new Set(prev).add(agentType));
      try {
        await loadQuotaForAgentType(agentType);
      } finally {
        setRefreshingAgentTypes((prev) => {
          const next = new Set(prev);
          next.delete(agentType);
          return next;
        });
      }
    },
    [loadQuotaForAgentType],
  );

  useEffect(() => {
    if (isLoading || hasRefreshedOnLoad) return;
    setHasRefreshedOnLoad(true);
    void runQuotaRefresh();
  }, [hasRefreshedOnLoad, isLoading, runQuotaRefresh]);

  useEffect(() => {
    if (isLoading) return;
    const interval = setInterval(() => {
      void runQuotaRefresh();
    }, 60_000);
    return () => clearInterval(interval);
  }, [isLoading, runQuotaRefresh]);

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
      <motion.div
        variants={containerVariants}
        initial={false}
        animate="show"
        className="grid gap-4 grid-cols-1 md:grid-cols-2"
        style={{ gridAutoRows: "minmax(200px, auto)" }}
      >
        <AnimatePresence mode="popLayout">
          {agents.map((agent) => {
            const agentType = agent.agentType;
            const providerType = agentTypeToProviderType(agentType);
            const isInstalled = agent.status !== "NotInstalled";
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
                  onRefresh={handleCardRefresh}
                  isRefreshing={refreshingAgentTypes.has(providerType)}
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
