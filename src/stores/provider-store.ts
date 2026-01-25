import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type {
  Provider,
  CreateProviderInput,
  UpdateProviderInput,
  AgentAuthStart,
  AgentQuota,
} from "@/types";
import { useRouterStore } from "./router-store";

interface ProviderState {
  providers: Provider[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchProviders: () => Promise<void>;
  createProvider: (input: CreateProviderInput) => Promise<Provider>;
  updateProvider: (id: string, input: UpdateProviderInput) => Promise<Provider>;
  deleteProvider: (id: string) => Promise<void>;
  testConnection: (id: string) => Promise<{ isConnected: boolean; latencyMs?: number; error?: string }>;
  authenticateAgentProvider: (id: string) => Promise<Provider>;
  fetchAgentQuota: (id: string) => Promise<AgentQuota>;
}

export const useProviderStore = create<ProviderState>((set) => ({
  providers: [],
  isLoading: false,
  error: null,

  fetchProviders: async () => {
    set({ isLoading: true, error: null });
    try {
      const providers = await invoke<Provider[]>("list_providers");
      set({ providers, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  createProvider: async (input: CreateProviderInput) => {
    try {
      const provider = await invoke<Provider>("create_provider", { input });
      set((state) => ({
        providers: [...state.providers, provider],
      }));
      return provider;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateProvider: async (id: string, input: UpdateProviderInput) => {
    try {
      const provider = await invoke<Provider>("update_provider", { id, input });
      set((state) => ({
        providers: state.providers.map((p) => (p.id === id ? provider : p)),
      }));
      return provider;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteProvider: async (id: string) => {
    try {
      await invoke("delete_provider", { id });
      set((state) => ({
        providers: state.providers.filter((p) => p.id !== id),
      }));
      // Refresh routing rules since backend removes rules referencing deleted provider
      useRouterStore.getState().fetchRules();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  testConnection: async (id: string) => {
    try {
      const result = await invoke<{ isConnected: boolean; latencyMs?: number; error?: string }>(
        "test_connection",
        { id }
      );
      // Update provider status based on connection test
      const status = result.isConnected ? "Connected" : "Disconnected";
      set((state) => ({
        providers: state.providers.map((p) =>
          p.id === id ? { ...p, status } : p
        ),
      }));
      return result;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  authenticateAgentProvider: async (id: string) => {
    try {
      const start = await invoke<AgentAuthStart>("start_agent_auth", { providerId: id });
      try {
        await openUrl(start.authUrl);
      } catch (error) {
        console.warn("Failed to open browser for auth:", error);
      }
      const provider = await invoke<Provider>("complete_agent_auth", { flowId: start.flowId });
      set((state) => ({
        providers: state.providers.map((p) => (p.id === id ? provider : p)),
      }));
      return provider;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  fetchAgentQuota: async (id: string) => {
    try {
      return await invoke<AgentQuota>("get_agent_quota", { providerId: id });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
