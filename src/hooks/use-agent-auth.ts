import { useEffect } from "react";
import { useAgentAuthStore } from "@/stores/agent-auth-store";

export function useAgentAuth() {
  const {
    accounts,
    isLoading,
    error,
    listAccounts,
    startAuth,
    completeAuth,
    getQuota,
    removeAuth,
  } = useAgentAuthStore();

  useEffect(() => {
    listAccounts();
  }, [listAccounts]);

  return {
    accounts,
    isLoading,
    error,
    startAuth,
    completeAuth,
    getQuota,
    removeAuth,
    refetch: listAccounts,
  };
}
