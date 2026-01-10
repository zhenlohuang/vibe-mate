import { useEffect } from "react";
import { useRouterStore } from "@/stores/router-store";

export function useRoutingRules() {
  const {
    rules,
    isLoading,
    error,
    fetchRules,
    createRule,
    updateRule,
    deleteRule,
    reorderRules,
  } = useRouterStore();

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  // Sort rules by priority
  const sortedRules = [...rules].sort((a, b) => a.priority - b.priority);

  return {
    rules: sortedRules,
    isLoading,
    error,
    createRule,
    updateRule,
    deleteRule,
    reorderRules,
    refetch: fetchRules,
  };
}

