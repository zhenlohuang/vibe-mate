import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Loader2, RefreshCw } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentQuotaCard } from "@/components/quota";
import { useAgentAuth } from "@/hooks/use-agent-auth";
import { AGENT_TYPES } from "@/lib/constants";
import type { AgentProviderType, AgentQuota } from "@/types";
import { Button } from "@/components/ui/button";
import { containerVariants, itemVariants } from "@/lib/animations";
import { cn } from "@/lib/utils";

export function QuotaPage() {
  const { accounts, isLoading, refetch, getQuota } = useAgentAuth();
  const [hasRefreshedOnLoad, setHasRefreshedOnLoad] = useState(false);
  const [quotaByAgentType, setQuotaByAgentType] = useState<Record<string, AgentQuota | null>>({});
  const [quotaErrorByAgentType, setQuotaErrorByAgentType] = useState<Record<string, string | null>>({});
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [activeGroupType, setActiveGroupType] = useState<string | null>(null);

  const accountByType = useMemo(() => {
    const map = new Map<AgentProviderType, (typeof accounts)[0]>();
    accounts.forEach((a) => map.set(a.agentType, a));
    return map;
  }, [accounts]);

  const orderedGroups = useMemo(() => {
    return AGENT_TYPES.map((agent) => ({
      type: agent.value as AgentProviderType,
      label: agent.label,
      account: accountByType.get(agent.value as AgentProviderType) ?? {
        agentType: agent.value as AgentProviderType,
        isAuthenticated: false,
        email: null,
      },
    }));
  }, [accountByType]);

  const resolvedGroupType =
    activeGroupType && orderedGroups.some((g) => g.type === activeGroupType)
      ? activeGroupType
      : orderedGroups[0]?.type ?? null;
  const activeGroup = resolvedGroupType
    ? orderedGroups.find((g) => g.type === resolvedGroupType) ?? null
    : null;

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
    const refreshable = orderedGroups.filter(
      (g) => g.account.isAuthenticated && g.account.agentType !== "GeminiCli",
    );
    await Promise.all(refreshable.map((g) => loadQuotaForAgentType(g.account.agentType)));
  }, [orderedGroups, loadQuotaForAgentType]);

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
        description="Track agent usage limits grouped by provider type."
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
      description="Track agent usage limits grouped by provider type."
    >
      <div className="mb-6 flex flex-wrap items-center gap-3">
        {orderedGroups.length > 1 ? (
          <div className="flex flex-wrap items-center gap-2 flex-1 min-w-[220px]">
            {orderedGroups.map((group) => {
              const isActive = group.type === resolvedGroupType;
              return (
                <button
                  key={group.type}
                  type="button"
                  onClick={() => setActiveGroupType(group.type)}
                  className={cn(
                    "flex items-center gap-2 rounded-full border px-3 py-1 text-[11px] font-medium transition-colors",
                    isActive
                      ? "border-primary/50 bg-primary/10 text-foreground"
                      : "border-border/60 bg-card/50 text-muted-foreground hover:text-foreground",
                  )}
                  aria-pressed={isActive}
                >
                  <span className="truncate">{group.label}</span>
                </button>
              );
            })}
          </div>
        ) : (
          <div className="flex-1 min-w-[220px]" />
        )}
        <Button
          size="sm"
          variant="secondary"
          className="ml-auto h-8 gap-2 px-3 text-[10px] uppercase tracking-wider"
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

      <div className="space-y-5">
        {activeGroup ? (
          <section className="space-y-3">
            <motion.div
              key={resolvedGroupType ?? "empty"}
              variants={containerVariants}
              initial={false}
              animate="show"
              className="grid gap-4 grid-cols-1"
            >
              <AnimatePresence mode="popLayout">
                <motion.div key={activeGroup.type} variants={itemVariants} layout initial={false}>
                  <AgentQuotaCard
                    account={activeGroup.account}
                    label={activeGroup.label}
                    quota={quotaByAgentType[activeGroup.type] ?? null}
                    quotaError={quotaErrorByAgentType[activeGroup.type] ?? null}
                    onRefresh={loadQuotaForAgentType}
                  />
                </motion.div>
              </AnimatePresence>
            </motion.div>
          </section>
        ) : (
          <div className="rounded-lg border border-dashed border-border/60 bg-card/30 px-6 py-10 text-center text-sm text-muted-foreground">
            No agent types configured.
          </div>
        )}
      </div>
    </MainContent>
  );
}
