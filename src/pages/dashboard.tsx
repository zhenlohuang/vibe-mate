import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Copy, Check, Settings2, Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { AgentCard } from "@/components/agents";
import { invoke } from "@tauri-apps/api/core";
import { useAgentAuth } from "@/hooks/use-agent-auth";
import { useAgents } from "@/hooks/use-agents";
import { useToast } from "@/hooks/use-toast";
import { useAppStore } from "@/stores/app-store";
import { agentTypeToProviderType } from "@/lib/constants";
import { getAgentName } from "@/lib/agents";
import type { AgentType, AgentProviderType, AgentQuota } from "@/types";
import { containerVariants, itemVariants } from "@/lib/animations";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

const API_ENDPOINTS = [
  { label: "OpenAI Compatible API", path: "/api/openai" },
  { label: "Anthropic Compatible API", path: "/api/anthropic" },
  { label: "Generic API", path: "/api" },
] as const;

export function DashboardPage() {
  const { proxyStatus } = useAppStore();
  const { accounts, isLoading: isAuthLoading, getQuota } = useAgentAuth();
  const { agents, isLoading: isAgentsLoading, refetch } = useAgents();
  const { toast } = useToast();
  const [hasRefreshedOnLoad, setHasRefreshedOnLoad] = useState(false);
  const [quotaByAgentType, setQuotaByAgentType] = useState<Record<string, AgentQuota | null>>({});
  const [quotaErrorByAgentType, setQuotaErrorByAgentType] = useState<Record<string, string | null>>({});
  const [refreshingAgentTypes, setRefreshingAgentTypes] = useState<Set<AgentProviderType>>(new Set());
  const [copiedUrl, setCopiedUrl] = useState<string | null>(null);
  const [featuredDialogOpen, setFeaturedDialogOpen] = useState(false);
  const [pendingFeatured, setPendingFeatured] = useState<string[]>([]);

  const isLoading = isAuthLoading || isAgentsLoading;
  const baseUrl = `http://localhost:${proxyStatus.port}`;
  const agentsToShow = useMemo(
    () => agents.filter((a) => a.featured !== false),
    [agents]
  );

  const copyUrl = useCallback(
    async (url: string) => {
      try {
        await navigator.clipboard.writeText(url);
        setCopiedUrl(url);
        toast({ title: "Copied", description: "URL copied to clipboard." });
        setTimeout(() => setCopiedUrl(null), 2000);
      } catch {
        toast({
          title: "Copy failed",
          description: "Could not copy to clipboard.",
          variant: "destructive",
        });
      }
    },
    [toast]
  );

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
    [getQuota]
  );

  const refreshableTypes = useMemo(() => {
    const installed = new Set<AgentProviderType>();
    agents.forEach((a) => {
      if (a.status !== "NotInstalled") {
        installed.add(agentTypeToProviderType(a.agentType));
      }
    });
    return [...installed].filter(
      (t) => accountByType.get(t)?.isAuthenticated && t !== "GeminiCli"
    );
  }, [agents, accountByType]);

  const refreshAllQuotas = useCallback(
    async (types?: AgentProviderType[]) => {
      const toRefresh = types ?? refreshableTypes;
      await Promise.all(toRefresh.map((t) => loadQuotaForAgentType(t)));
    },
    [refreshableTypes, loadQuotaForAgentType]
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
    [loadQuotaForAgentType]
  );

  useEffect(() => {
    if (isLoading || hasRefreshedOnLoad) return;
    setHasRefreshedOnLoad(true);
    void runQuotaRefresh();
  }, [hasRefreshedOnLoad, isLoading, runQuotaRefresh]);

  useEffect(() => {
    if (isLoading) return;
    const interval = setInterval(() => void runQuotaRefresh(), 60_000);
    return () => clearInterval(interval);
  }, [isLoading, runQuotaRefresh]);

  const openFeaturedDialog = useCallback(() => {
    const current = agents
      .filter((a) => a.featured !== false)
      .map((a) => a.agentType);
    setPendingFeatured(agents.length > 0 ? current : []);
    setFeaturedDialogOpen(true);
  }, [agents]);

  const togglePending = useCallback((agentType: AgentType) => {
    setPendingFeatured((prev) =>
      prev.includes(agentType)
        ? prev.filter((id) => id !== agentType)
        : [...prev, agentType]
    );
  }, []);

  const saveFeatured = useCallback(async () => {
    try {
      await Promise.all(
        agents.map((a) =>
          invoke("set_coding_agent_featured", {
            agentType: a.agentType,
            featured: pendingFeatured.includes(a.agentType),
          })
        )
      );
      await refetch();
      setFeaturedDialogOpen(false);
      toast({ title: "Saved", description: "Dashboard agents updated." });
    } catch (e) {
      toast({
        title: "Failed to save",
        description: String(e),
        variant: "destructive",
      });
    }
  }, [agents, pendingFeatured, refetch, toast]);

  return (
    <MainContent title="Dashboard" description="Your mate for Vibe Coding">
      <div className="space-y-8">
        {/* API 调用地址 */}
        <section>
          <h2 className="text-sm font-medium text-foreground mb-2">API Proxy</h2>
          <Card className="overflow-hidden">
            <CardContent className="p-2">
              <div className="flex flex-col gap-0.5">
                {API_ENDPOINTS.map(({ label, path }) => {
                  const url = `${baseUrl}${path}`;
                  const isCopied = copiedUrl === url;
                  return (
                    <div
                      key={path}
                      className="flex items-center gap-2 min-h-7 px-1.5 rounded hover:bg-secondary/50"
                    >
                      <span className="text-[11px] text-muted-foreground w-24 shrink-0">
                        {label.replace(" Compatible API", "")}
                      </span>
                      <code className="text-[11px] text-foreground truncate flex-1 min-w-0">
                        {url}
                      </code>
                      <button
                        type="button"
                        onClick={() => copyUrl(url)}
                        className="shrink-0 p-1 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
                        title="Copy"
                      >
                        {isCopied ? (
                          <Check className="h-3 w-3 text-success" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                      </button>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </section>

        {/* Coding Agent 列表 */}
        <section>
          <div className="flex items-center justify-between gap-2 mb-3">
            <h2 className="text-sm font-medium text-foreground">Featured Coding Agents</h2>
            <Button
              variant="ghost"
              size="icon"
              onClick={openFeaturedDialog}
              title="选择显示在 Dashboard 上的 Agent"
            >
              <Settings2 className="h-4 w-4" />
            </Button>
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-6 w-6 animate-spin text-primary" />
            </div>
          ) : (
            <motion.div
              variants={containerVariants}
              initial={false}
              animate="show"
              className="grid gap-4 grid-cols-1 md:grid-cols-2"
              style={{ gridAutoRows: "minmax(200px, auto)" }}
            >
              <AnimatePresence mode="popLayout">
                {agentsToShow.map((agent) => {
                  const agentType = agent.agentType;
                  const providerType = agentTypeToProviderType(agentType);
                  const label = getAgentName(agentType);
                  const account =
                    accountByType.get(providerType) ?? ({
                      agentType: providerType,
                      isAuthenticated: false,
                      email: null,
                    });
                  return (
                    <motion.div
                      key={agentType}
                      variants={itemVariants}
                      layout
                      initial={false}
                    >
                      <AgentCard
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
          )}
        </section>
      </div>

      <Dialog open={featuredDialogOpen} onOpenChange={setFeaturedDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>显示在 Dashboard 上的 Agent</DialogTitle>
          </DialogHeader>
          <div className="space-y-2 py-2">
            {agents.map((agent) => (
              <label
                key={agent.agentType}
                className="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1.5 hover:bg-secondary/50"
              >
                <input
                  type="checkbox"
                  checked={pendingFeatured.includes(agent.agentType)}
                  onChange={() => togglePending(agent.agentType)}
                  className="rounded border-border"
                />
                <span className="text-sm">{getAgentName(agent.agentType)}</span>
              </label>
            ))}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setFeaturedDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={saveFeatured}>Save</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </MainContent>
  );
}
