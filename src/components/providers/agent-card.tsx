import { motion } from "motion/react";
import { useEffect, useMemo, useState } from "react";
import { LogIn, RefreshCw, Loader2 } from "lucide-react";
import type { Provider, AgentQuota } from "@/types";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ProviderLogo } from "./provider-logo";
import { useProviderStore } from "@/stores/provider-store";
import { useToast } from "@/hooks/use-toast";

interface AgentCardProps {
  provider: Provider;
}

function getStatusConfig(status: Provider["status"], isDefault: boolean) {
  if (isDefault) {
    return {
      label: "ACTIVE",
      className: "bg-success/20 text-success",
      dotClassName: "bg-success",
    };
  }
  switch (status) {
    case "Connected":
      return {
        label: "STANDBY",
        className: "bg-warning/20 text-warning",
        dotClassName: "bg-warning",
      };
    case "Disconnected":
      return {
        label: "INACTIVE",
        className: "bg-muted text-muted-foreground",
        dotClassName: "bg-muted-foreground",
      };
    case "Error":
      return {
        label: "ERROR",
        className: "bg-destructive/20 text-destructive",
        dotClassName: "bg-destructive",
      };
    default:
      return {
        label: "INACTIVE",
        className: "bg-muted text-muted-foreground",
        dotClassName: "bg-muted-foreground",
      };
  }
}

export function AgentCard({ provider }: AgentCardProps) {
  const isLoggedIn = provider.authPath !== null && provider.authPath !== undefined;
  const statusConfig = isLoggedIn
    ? {
        label: "ACTIVE",
        className: "bg-success/20 text-success",
        dotClassName: "bg-success",
      }
    : getStatusConfig(provider.status, provider.isDefault);
  const isAuthSupported = provider.type === "Codex";
  const authenticateAgentProvider = useProviderStore(
    (state) => state.authenticateAgentProvider
  );
  const fetchAgentQuota = useProviderStore((state) => state.fetchAgentQuota);
  const { toast } = useToast();
  const [isAuthLoading, setIsAuthLoading] = useState(false);
  const [isQuotaLoading, setIsQuotaLoading] = useState(false);
  const [quota, setQuota] = useState<AgentQuota | null>(null);
  const [quotaError, setQuotaError] = useState<string | null>(null);
  const quotaLabels =
    provider.type === "Codex"
      ? { session: "5h limit", week: "Weekly limit" }
      : { session: "Current Session", week: "Current Week" };

  const sessionLeftPercentage = useMemo(() => {
    const used = quota?.sessionUsedPercent ?? 0;
    return Math.min(100, Math.max(0, 100 - used));
  }, [quota?.sessionUsedPercent]);
  const weekLeftPercentage = useMemo(() => {
    const used = quota?.weekUsedPercent ?? 0;
    return Math.min(100, Math.max(0, 100 - used));
  }, [quota?.weekUsedPercent]);

  const formatResetAt = (timestamp?: number | null) => {
    if (!timestamp) return "—";
    return new Date(timestamp * 1000).toLocaleString();
  };

  const loadQuota = async () => {
    setIsQuotaLoading(true);
    setQuotaError(null);
    try {
      const data = await fetchAgentQuota(provider.id);
      setQuota(data);
    } catch (error) {
      setQuotaError(String(error));
    } finally {
      setIsQuotaLoading(false);
    }
  };

  useEffect(() => {
    if (!isAuthSupported) {
      setQuota(null);
      setQuotaError("Usage is not available for this agent yet.");
      return;
    }
    if (isLoggedIn) {
      loadQuota();
    } else {
      setQuota(null);
      setQuotaError(null);
    }
  }, [isLoggedIn, isAuthSupported, provider.id, provider.authPath]);

  const handleLogin = async () => {
    setIsAuthLoading(true);
    try {
      await authenticateAgentProvider(provider.id);
      toast({
        title: "Authentication complete",
        description: `${provider.name} is now connected.`,
      });
      await loadQuota();
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
    >
      <Card
        className={cn(
          "provider-card relative overflow-hidden",
          provider.isDefault && "ring-1 ring-primary/50"
        )}
      >
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <ProviderLogo type={provider.type} />
              <span className="text-sm font-semibold truncate">{provider.name}</span>
            </div>
            <div
              className={cn(
                "flex items-center gap-1 rounded-full px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wider shrink-0",
                statusConfig.className
              )}
            >
              <div className={cn("h-1 w-1 rounded-full", statusConfig.dotClassName)} />
              {statusConfig.label}
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-3">
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
              <div className="flex items-center justify-between text-[10px] text-muted-foreground">
                <div className="flex flex-col gap-1">
                  <span className="uppercase tracking-wider">
                    {quota?.planType ? `${quota.planType} plan` : "Plan unknown"}
                  </span>
                  {provider.authEmail ? (
                    <span className="font-mono text-[9px] text-muted-foreground/80">
                      {provider.authEmail}
                    </span>
                  ) : null}
                </div>
                <button
                  type="button"
                  onClick={loadQuota}
                  className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[9px] uppercase tracking-wider transition-colors hover:bg-secondary"
                  disabled={isQuotaLoading}
                >
                  {isQuotaLoading ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3 w-3" />
                  )}
                  Refresh
                </button>
              </div>

              {quotaError ? (
                <div className="rounded-md border border-destructive/40 bg-destructive/10 px-2 py-2 text-[10px] text-destructive">
                  {quotaError}
                </div>
              ) : (
                <>
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-[10px]">
                      <span className="font-medium uppercase tracking-wider text-muted-foreground">
                        {quotaLabels.session}
                      </span>
                      <span className="font-mono text-foreground/80">
                        {sessionLeftPercentage.toFixed(1)}% left
                      </span>
                    </div>
                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${sessionLeftPercentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground">
                      Resets: {formatResetAt(quota?.sessionResetAt)}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-[10px]">
                      <span className="font-medium uppercase tracking-wider text-muted-foreground">
                        {quotaLabels.week}
                      </span>
                      <span className="font-mono text-foreground/80">
                        {weekLeftPercentage.toFixed(1)}% left
                      </span>
                    </div>
                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-secondary/50">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${weekLeftPercentage}%` }}
                      />
                    </div>
                    <div className="text-[9px] text-muted-foreground">
                      Resets: {formatResetAt(quota?.weekResetAt)}
                    </div>
                  </div>
                </>
              )}
            </>
          )}

          <div className="pt-1" />
        </CardContent>
      </Card>
    </motion.div>
  );
}
