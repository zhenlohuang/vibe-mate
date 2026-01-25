import { useEffect } from "react";
import { useProviderStore } from "@/stores/provider-store";

export function useProviders() {
  const {
    providers,
    isLoading,
    error,
    fetchProviders,
    createProvider,
    updateProvider,
    deleteProvider,
    testConnection,
    authenticateAgentProvider,
    fetchAgentQuota,
  } = useProviderStore();

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  return {
    providers,
    isLoading,
    error,
    createProvider,
    updateProvider,
    deleteProvider,
    testConnection,
    authenticateAgentProvider,
    fetchAgentQuota,
    refetch: fetchProviders,
  };
}
