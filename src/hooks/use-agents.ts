import { useEffect } from "react";
import { useAgentStore } from "@/stores/agent-store";

export function useAgents() {
  const {
    agents,
    isLoading,
    error,
    discoverAgents,
    checkStatus,
    openLogin,
    getVersion,
  } = useAgentStore();

  useEffect(() => {
    discoverAgents();
  }, [discoverAgents]);

  return {
    agents,
    isLoading,
    error,
    checkStatus,
    openLogin,
    getVersion,
    refetch: discoverAgents,
  };
}

