import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentConfigItem, AgentType, CodingAgent } from "@/types";
import { getAgentName, getDefaultConfigPath } from "@/lib/agents";

function normalizeConfig(config: AgentConfigItem[]): AgentConfigItem[] {
  return config.map((item) => ({
    ...item,
    configFile: item.configFile || getDefaultConfigPath(item.type),
  }));
}

export function useAgents() {
  const [agentsConfig, setAgentsConfig] = useState<AgentConfigItem[] | null>(null);
  const agentsConfigRef = useRef<AgentConfigItem[] | null>(null);

  // Keep ref in sync with state
  useEffect(() => {
    agentsConfigRef.current = agentsConfig;
  }, [agentsConfig]);

  const fetchConfig = useCallback(async () => {
    const config = await invoke<AgentConfigItem[]>("get_agents_config");
    const normalized = normalizeConfig(config);
    setAgentsConfig(normalized);
    return normalized;
  }, []);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const agents: CodingAgent[] = agentsConfig
    ? agentsConfig.map((item) => ({
        agentType: item.type,
        name: getAgentName(item.type),
        configPath: item.configFile || getDefaultConfigPath(item.type),
      }))
    : [];

  const addAgent = useCallback(async (agentType: AgentType) => {
    const currentConfig = agentsConfigRef.current;
    if (!currentConfig) return;
    if (currentConfig.some((item) => item.type === agentType)) {
      return;
    }

    const nextConfig = [
      ...currentConfig,
      { type: agentType, configFile: getDefaultConfigPath(agentType) },
    ];

    const result = await invoke<AgentConfigItem[]>("update_agents_config", {
      input: { agents: nextConfig }
    });
    setAgentsConfig(normalizeConfig(result));
  }, []);

  const removeAgent = useCallback(async (agentType: AgentType) => {
    const currentConfig = agentsConfigRef.current;
    if (!currentConfig) return;

    const nextConfig = currentConfig.filter((item) => item.type !== agentType);

    const result = await invoke<AgentConfigItem[]>("update_agents_config", {
      input: { agents: nextConfig }
    });
    setAgentsConfig(normalizeConfig(result));
  }, []);

  const updateAgentConfigPath = useCallback(async (agentType: AgentType, configPath: string) => {
    const currentConfig = agentsConfigRef.current;
    if (!currentConfig) return;

    const resolvedPath = configPath.trim() || getDefaultConfigPath(agentType);
    const nextConfig = currentConfig.map((item) =>
      item.type === agentType
        ? { ...item, configFile: resolvedPath }
        : item
    );

    const result = await invoke<AgentConfigItem[]>("update_agents_config", {
      input: { agents: nextConfig }
    });
    setAgentsConfig(normalizeConfig(result));
  }, []);

  return {
    agents,
    isLoading: !agentsConfig,
    addAgent,
    removeAgent,
    updateAgentConfigPath,
    refetch: fetchConfig,
  };
}
