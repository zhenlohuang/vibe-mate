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
    setDefaultProvider,
    testConnection,
  } = useProviderStore();

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const defaultProvider = providers.find((p) => p.isDefault);

  return {
    providers,
    defaultProvider,
    isLoading,
    error,
    createProvider,
    updateProvider,
    deleteProvider,
    setDefaultProvider,
    testConnection,
    refetch: fetchProviders,
  };
}

