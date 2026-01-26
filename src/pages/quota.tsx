import { useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Loader2, RefreshCw } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentQuotaCard } from "@/components/quota";
import { useProviders } from "@/hooks/use-providers";
import { AGENT_TYPES } from "@/lib/constants";
import type { AgentProviderType, Provider } from "@/types";
import { Button } from "@/components/ui/button";
import { containerVariants, itemVariants } from "@/lib/animations";
import { cn } from "@/lib/utils";

interface QuotaGroup {
  type: AgentProviderType | string;
  label: string;
  providers: Provider[];
}

export function QuotaPage() {
  const { providers, isLoading, refetch } = useProviders();
  const [refreshToken, setRefreshToken] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [activeGroupType, setActiveGroupType] = useState<string | null>(null);

  const agentProviders = useMemo(
    () => providers.filter((provider) => provider.category === "Agent"),
    [providers],
  );

  const groupedProviders = useMemo(() => {
    const map = new Map<string, Provider[]>();
    agentProviders.forEach((provider) => {
      const key = provider.type;
      const group = map.get(key) ?? [];
      group.push(provider);
      map.set(key, group);
    });
    return map;
  }, [agentProviders]);

  const orderedGroups = useMemo<QuotaGroup[]>(() => {
    const groups: QuotaGroup[] = AGENT_TYPES.map((agent) => ({
      type: agent.value as AgentProviderType,
      label: agent.label,
      providers: groupedProviders.get(agent.value) ?? [],
    })).filter((group) => group.providers.length > 0);

    const knownTypes = new Set(AGENT_TYPES.map((agent) => agent.value));
    groupedProviders.forEach((providers, type) => {
      if (!knownTypes.has(type)) {
        groups.push({ type, label: type, providers });
      }
    });

    return groups;
  }, [groupedProviders]);

  const resolvedGroupType =
    activeGroupType && orderedGroups.some((group) => group.type === activeGroupType)
      ? activeGroupType
      : orderedGroups[0]?.type ?? null;
  const activeGroup = resolvedGroupType
    ? orderedGroups.find((group) => group.type === resolvedGroupType) ?? null
    : null;

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refetch();
      setRefreshToken((prev) => prev + 1);
    } finally {
      setIsRefreshing(false);
    }
  };

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
                  <span
                    className={cn(
                      "flex h-5 min-w-[20px] items-center justify-center rounded-full px-1 text-[10px] font-semibold",
                      isActive ? "bg-primary text-primary-foreground" : "bg-secondary text-foreground",
                    )}
                  >
                    {group.providers.length}
                  </span>
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

      {agentProviders.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border/60 bg-card/30 px-6 py-10 text-center text-sm text-muted-foreground">
          No agent providers added yet. Add an agent in Providers to see quota.
        </div>
      ) : (
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
                  {activeGroup.providers.map((provider) => (
                    <motion.div key={provider.id} variants={itemVariants} layout initial={false}>
                      <AgentQuotaCard provider={provider} refreshToken={refreshToken} />
                    </motion.div>
                  ))}
                </AnimatePresence>
              </motion.div>
            </section>
          ) : (
            <div className="rounded-lg border border-dashed border-border/60 bg-card/30 px-6 py-10 text-center text-sm text-muted-foreground">
              No providers in this group yet.
            </div>
          )}
        </div>
      )}
    </MainContent>
  );
}
