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
    refetch: fetchProviders,
  };
}
