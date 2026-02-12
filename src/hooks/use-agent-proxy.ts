import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@/hooks/use-toast";
import { getAgentName } from "@/lib/agents";
import { isProxyCompatibleAgentType } from "@/lib/constants";
import type { AgentType, CodingAgent } from "@/types";

export function useAgentProxy(agents: CodingAgent[], isLoading: boolean) {
  const { toast } = useToast();
  const [proxyStatusByType, setProxyStatusByType] = useState<Record<string, boolean>>({});

  useEffect(() => {
    if (isLoading) return;

    const supportedAgentTypes = agents
      .map((agent) => agent.agentType)
      .filter(isProxyCompatibleAgentType);

    if (supportedAgentTypes.length === 0) return;

    let cancelled = false;
    void Promise.all(
      supportedAgentTypes.map(async (agentType) => {
        try {
          const enabled = await invoke<boolean>("is_agent_proxy_enabled", { agentType });
          return [agentType, enabled] as const;
        } catch {
          return [agentType, false] as const;
        }
      }),
    ).then((results) => {
      if (cancelled) return;
      setProxyStatusByType((prev) => {
        const next = { ...prev };
        results.forEach(([agentType, enabled]) => {
          next[agentType] = enabled;
        });
        return next;
      });
    });

    return () => {
      cancelled = true;
    };
  }, [agents, isLoading]);

  const handleProxyToggle = useCallback(
    async (agentType: AgentType, enabled: boolean) => {
      try {
        await invoke("set_agent_proxy_enabled", { agentType, enabled });
        setProxyStatusByType((prev) => ({ ...prev, [agentType]: enabled }));
        toast({
          title: enabled ? "VibeMate Proxy enabled" : "Proxy disabled",
          description: `${getAgentName(agentType)} configuration has been updated.`,
        });
      } catch (error) {
        toast({
          title: "Failed to update proxy",
          description: String(error),
          variant: "destructive",
        });
      }
    },
    [toast],
  );

  const getProxyToggleProps = useCallback(
    (agentType: AgentType) => {
      if (!isProxyCompatibleAgentType(agentType)) {
        return {};
      }
      return {
        proxyEnabled: proxyStatusByType[agentType] ?? false,
        onProxyToggle: (enabled: boolean) => {
          void handleProxyToggle(agentType, enabled);
        },
      };
    },
    [handleProxyToggle, proxyStatusByType],
  );

  return {
    getProxyToggleProps,
    isProxyCompatibleAgentType,
    proxyStatusByType,
  };
}
