import { motion } from "motion/react";
import { useMemo, useState } from "react";
import { LogIn, Loader2, RefreshCw } from "lucide-react";
import type { AgentAccountInfo, AgentQuota, AgentQuotaEntry, AgentProviderType } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProviderLogo } from "@/components/providers/provider-logo";
import { useAgentAuthStore } from "@/stores/agent-auth-store";
import { useToast } from "@/hooks/use-toast";

interface AgentQuotaCardProps {
  account: AgentAccountInfo;
  label: string;
  quota?: AgentQuota | null;
  quotaError?: string | null;
  onRefresh?: (agentType: AgentProviderType) => Promise<void> | void;
}

function getStatusConfig(authenticated: boolean) {
  return authenticated
    ? { label: "ACTIVE", className: "bg-success/20 text-success", dotClassName: "bg-success" }
    : { label: "INACTIVE", className: "bg-muted text-muted-foreground", dotClassName: "bg-muted-foreground" };
}

export function AgentQuotaCard({
  account,
  label,
  quota,
  quotaError,
  onRefresh,
}: AgentQuotaCardProps) {
  const isLoggedIn = account.isAuthenticated;
  const statusConfig = getStatusConfig(isLoggedIn);
  const agentType = account.agentType;
  const isAuthSupported = true;
  const isQuotaSupported = agentType !== "GeminiCli";
  const startAuth = useAgentAuthStore((state) => state.startAuth);
  const completeAuth = useAgentAuthStore((state) => state.completeAuth);
  const { toast } = useToast();
  const [isAuthLoading, setIsAuthLoading] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);

  const resolvedQuotaError = !isAuthSupported
    ? "Usage is not available for this agent yet."
    : quotaError ?? null;
  const quotaLabels = useMemo(() => {
    switch (agentType) {
      case "Codex":
        return { session: "5h limit", week: "Weekly limit" };
      case "ClaudeCode":
        return { session: "Current session", week: "Current week" };
      case "Antigravity":
        return { session: "Primary model", week: "Secondary model" };
      default:
        return { session: "Current Session", week: "Current Week" };
    }
  }, [agentType]);
  const sessionUsedPercentage = useMemo(() => {
    const used = quota?.sessionUsedPercent ?? 0;
    return Math.min(100, Math.max(0, used));
  }, [quota?.sessionUsedPercent]);
  const weekUsedPercentage = useMemo(() => {
    const used = quota?.weekUsedPercent ?? 0;
    return Math.min(100, Math.max(0, used));
  }, [quota?.weekUsedPercent]);
  const quotaEntries = useMemo<AgentQuotaEntry[]>(
    () => quota?.entries?.filter(Boolean) ?? [],
    [quota?.entries],
  );
  const hasEntries = agentType === "Antigravity" && quotaEntries.length > 0;
  const resetPrefix = agentType === "ClaudeCode" ? "Resets" : "Resets:";
  const entryDisplayLimit = agentType === "Antigravity" ? 2 : quotaEntries.length;
  const displayedEntries = useMemo(
    () => (isExpanded ? quotaEntries : quotaEntries.slice(0, entryDisplayLimit)),
    [quotaEntries, entryDisplayLimit, isExpanded],
  );
  const remainingEntryCount = Math.max(0, quotaEntries.length - entryDisplayLimit);
  const showExpandToggle =
    agentType === "Antigravity" && hasEntries && (remainingEntryCount > 0 || isExpanded);
  const entriesContainerClass = "space-y-3";

  const formatUsageText = (used: number) => `${used.toFixed(1)}% used`;

  const formatResetAt = (timestamp?: number | null) => {
    if (!timestamp) return "—";
    const date = new Date(timestamp * 1000);
    if (agentType === "ClaudeCode") {
      const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const now = new Date();
      const sameDay = date.toDateString() === now.toDateString();
      const formatter = new Intl.DateTimeFormat(
        undefined,
        sameDay
          ? { hour: "numeric", minute: "2-digit" }
          : { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" },
      );
      return `${formatter.format(date)} (${timeZone})`;
    }
    return date.toLocaleString();
  };

  const handleRefresh = async () => {
    if (!isQuotaSupported) return;
    await onRefresh?.(agentType);
  };

  const handleLogin = async () => {
    setIsAuthLoading(true);
    try {
      const start = await startAuth(agentType);
      await completeAuth(start.flowId);
      toast({
        title: "Authentication complete",
        description: `${label} is now connected.`,
      });
      await handleRefresh();
    } catch (error) {
      toast({
        title: "Authentication failed",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsAuthLoading(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      whileHover={{ y: -2 }}
      transition={{ duration: 0.2 }}
      className="h-full"
    >
      <Card className={cn("provider-card relative flex h-full flex-col overflow-hidden")}>
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <ProviderLogo type={agentType} />
              <div className="flex flex-col min-w-0">
                <span className="text-sm font-semibold truncate">{label}</span>
                {isLoggedIn && account.email ? (
                  <span className="text-[10px] text-muted-foreground truncate">{account.email}</span>
                ) : null}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <div
                className={cn(
                  "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                  statusConfig.className,
                )}
              >
                <div className={cn("h-1 w-1 rounded-full", statusConfig.dotClassName)} />
                {statusConfig.label}
              </div>
              {isLoggedIn && isQuotaSupported ? (
                <button
                  type="button"
                  onClick={handleRefresh}
                  className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                  aria-label={`Refresh ${label}`}
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                </button>
              ) : null}
            </div>
          </div>
        </CardHeader>

        <CardContent className="flex flex-1 flex-col space-y-3">
          {!isLoggedIn ? (
            <div className="py-4">
              <Button
                size="sm"
                variant="outline"
                className="w-full"
                onClick={handleLogin}
                disabled={!isAuthSupported || isAuthLoading}
              >
                {isAuthLoading ? (
                  <>
                    <Loader2 className="h-3.5 w-3.5 mr-2 animate-spin" />
                    Authenticating...
                  </>
                ) : isAuthSupported ? (
                  <>
                    <LogIn className="h-3.5 w-3.5 mr-2" />
                    Login
                  </>
                ) : (
                  "Not supported"
                )}
              </Button>
            </div>
          ) : (
            <>
              {!isQuotaSupported ? (
                <div className="rounded-md border border-border/60 bg-muted/40 px-2 py-2 text-[10px] text-muted-foreground">
                  Usage is not available for this agent yet.
                </div>
              ) : resolvedQuotaError ? (
                <div className="rounded-md border border-destructive/40 bg-destructive/10 px-2 py-2 text-[10px] text-destructive">
                  {resolvedQuotaError}
                </div>
              ) : hasEntries ? (
                <>
                  <div className={entriesContainerClass}>
                    {displayedEntries.map((entry) => {
                      const used = Math.min(100, Math.max(0, entry.usedPercent ?? 0));
                      return (
                        <div key={entry.label} className="space-y-2">
                          <div className="flex items-center justify-between text-[10px]">
                            <span className="font-medium uppercase tracking-wider text-muted-foreground">
                              {entry.label}
                            </span>
                            <span className="font-mono text-foreground/80">
                              {formatUsageText(used)}
                            </span>
                          </div>
                          <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                            <div className="h-full bg-primary transition-all" style={{ width: `${used}%` }} />
                          </div>
                          <div className="text-[9px] text-muted-foreground">
                            {resetPrefix} {formatResetAt(entry.resetAt)}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                  {showExpandToggle ? (
                    <div className="pt-1">
                      <button
                        type="button"
                        onClick={() => setIsExpanded((prev) => !prev)}
                        className="text-[9px] uppercase tracking-wider text-muted-foreground hover:text-foreground transition-colors"
                      >
                        {isExpanded ? "Show less" : `${remainingEntryCount}+ more models`}
                      </button>
                    </div>
                  ) : null}
                  {quota?.note ? (
                    <div className="rounded-md border border-border/60 bg-muted/40 px-2 py-2 text-[10px] text-muted-foreground">
                      {quota.note}
                    </div>
                  ) : null}
                </>
              ) : (
                <>
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-[10px]">
                      <span className="font-medium uppercase tracking-wider text-muted-foreground">
                        {quotaLabels.session}
                      </span>
                      <span className="font-mono text-foreground/80">
                        {formatUsageText(sessionUsedPercentage)}
                      </span>
                    </div>
                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${sessionUsedPercentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground">
                      {resetPrefix} {formatResetAt(quota?.sessionResetAt)}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-[10px]">
                      <span className="font-medium uppercase tracking-wider text-muted-foreground">
                        {quotaLabels.week}
                      </span>
                      <span className="font-mono text-foreground/80">
                        {formatUsageText(weekUsedPercentage)}
                      </span>
                    </div>
                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${weekUsedPercentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground">
                      {resetPrefix} {formatResetAt(quota?.weekResetAt)}
                    </div>
                  </div>
                  {quota?.note ? (
                    <div className="rounded-md border border-border/60 bg-muted/40 px-2 py-2 text-[10px] text-muted-foreground">
                      {quota.note}
                    </div>
                  ) : null}
                </>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </motion.div>
  );
}
