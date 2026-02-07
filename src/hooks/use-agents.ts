import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { CodingAgent } from "@/types";

export function useAgents() {
  const [agents, setAgents] = useState<CodingAgent[] | null>(null);

  const fetchAgents = useCallback(async () => {
    const list = await invoke<CodingAgent[]>("get_coding_agents");
    setAgents(list);
    return list;
  }, []);

  const refetchAgents = useCallback(async () => {
    const list = await invoke<CodingAgent[]>("refresh_coding_agents");
    setAgents(list);
    return list;
  }, []);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  const checkStatus = useCallback(async (agentType: string) => {
    const agent = await invoke<CodingAgent>("check_status", { agentType });
    return agent;
  }, []);

  return {
    agents: agents || [],
    isLoading: !agents,
    checkStatus,
    refetch: refetchAgents,
    fetchAgents,
  };
}
