import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Loader2, RefreshCw } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentQuotaCard } from "@/components/quota";
import { useAgentAuth } from "@/hooks/use-agent-auth";
import { useAgents } from "@/hooks/use-agents";
import { AGENT_TYPES, agentTypeToProviderType } from "@/lib/constants";
import type { AgentProviderType, AgentQuota } from "@/types";
import { Button } from "@/components/ui/button";
import { containerVariants, itemVariants } from "@/lib/animations";

export function QuotaPage() {
  const { accounts, isLoading: isAuthLoading, refetch, getQuota } = useAgentAuth();
  const { agents, isLoading: isAgentsLoading } = useAgents();
  const [hasRefreshedOnLoad, setHasRefreshedOnLoad] = useState(false);
  const [quotaByAgentType, setQuotaByAgentType] = useState<Record<string, AgentQuota | null>>({});
  const [quotaErrorByAgentType, setQuotaErrorByAgentType] = useState<Record<string, string | null>>({});
  const [isRefreshing, setIsRefreshing] = useState(false);

  const isLoading = isAuthLoading || isAgentsLoading;

  // Build a set of installed agent provider types from discovered agents
  const installedProviderTypes = useMemo(() => {
    const set = new Set<AgentProviderType>();
    for (const agent of agents) {
      if (agent.status !== "NotInstalled") {
        set.add(agentTypeToProviderType(agent.agentType));
      }
    }
    return set;
  }, [agents]);

  const accountByType = useMemo(() => {
    const map = new Map<AgentProviderType, (typeof accounts)[0]>();
    accounts.forEach((a) => map.set(a.agentType, a));
    return map;
  }, [accounts]);

  // Only show agents that are installed
  const visibleAgents = useMemo(() => {
    return AGENT_TYPES.filter((agent) =>
      installedProviderTypes.has(agent.value as AgentProviderType),
    ).map((agent) => ({
      type: agent.value as AgentProviderType,
      label: agent.label,
      account: accountByType.get(agent.value as AgentProviderType) ?? {
        agentType: agent.value as AgentProviderType,
        isAuthenticated: false,
        email: null,
      },
    }));
  }, [installedProviderTypes, accountByType]);

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
    const refreshable = visibleAgents.filter(
      (g) => g.account.isAuthenticated && g.account.agentType !== "GeminiCli",
    );
    await Promise.all(refreshable.map((g) => loadQuotaForAgentType(g.account.agentType)));
  }, [visibleAgents, loadQuotaForAgentType]);

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

  if (isLoading) {
    return (
      <MainContent
        title="Quota"
        description="Track agent usage limits."
      >
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
        </div>
      </MainContent>
    );
  }

  return (
    <MainContent
      title="Quota"
      description="Track agent usage limits."
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

      {visibleAgents.length > 0 ? (
        <motion.div
          variants={containerVariants}
          initial={false}
          animate="show"
          className="grid gap-4 grid-cols-1 md:grid-cols-2"
        >
          <AnimatePresence mode="popLayout">
            {visibleAgents.map((agent) => (
              <motion.div key={agent.type} variants={itemVariants} layout initial={false}>
                <AgentQuotaCard
                  account={agent.account}
                  label={agent.label}
                  quota={quotaByAgentType[agent.type] ?? null}
                  quotaError={quotaErrorByAgentType[agent.type] ?? null}
                  onRefresh={loadQuotaForAgentType}
                />
              </motion.div>
            ))}
          </AnimatePresence>
        </motion.div>
      ) : (
        <div className="rounded-lg border border-dashed border-border/60 bg-card/30 px-6 py-10 text-center text-sm text-muted-foreground">
          No coding agents detected. Install a supported agent (Claude Code, Codex, or Gemini CLI) to see quota information.
        </div>
      )}
    </MainContent>
  );
}
