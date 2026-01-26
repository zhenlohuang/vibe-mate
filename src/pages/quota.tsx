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

interface QuotaGroup {
  type: AgentProviderType | string;
  label: string;
  providers: Provider[];
}

export function QuotaPage() {
  const { providers, isLoading, refetch } = useProviders();
  const [refreshToken, setRefreshToken] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);

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
      <div className="mb-6 flex items-center justify-between gap-4">
        <div className="text-xs text-muted-foreground">
          Login and monitor usage for each agent provider.
        </div>
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

      {agentProviders.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border/60 bg-card/30 px-6 py-10 text-center text-sm text-muted-foreground">
          No agent providers added yet. Add an agent in Providers to see quota.
        </div>
      ) : (
        <div className="space-y-8">
          {orderedGroups.map((group) => (
            <section key={group.type} className="space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-sm font-semibold">{group.label}</h2>
                  <p className="text-[11px] text-muted-foreground">
                    {group.providers.length} provider
                    {group.providers.length === 1 ? "" : "s"}
                  </p>
                </div>
              </div>
              <motion.div
                variants={containerVariants}
                initial="hidden"
                animate="show"
                className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5"
              >
                <AnimatePresence mode="popLayout">
                  {group.providers.map((provider) => (
                    <motion.div key={provider.id} variants={itemVariants} layout>
                      <AgentQuotaCard provider={provider} refreshToken={refreshToken} />
                    </motion.div>
                  ))}
                </AnimatePresence>
              </motion.div>
            </section>
          ))}
        </div>
      )}
    </MainContent>
  );
}
