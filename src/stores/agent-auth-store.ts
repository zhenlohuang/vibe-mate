import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type {
  AgentAccountInfo,
  AgentAuthStart,
  AgentProviderType,
  AgentQuota,
} from "@/types";

interface AgentAuthState {
  accounts: AgentAccountInfo[];
  isLoading: boolean;
  error: string | null;

  listAccounts: () => Promise<void>;
  startAuth: (agentType: AgentProviderType) => Promise<AgentAuthStart>;
  completeAuth: (flowId: string) => Promise<AgentAccountInfo>;
  getQuota: (agentType: AgentProviderType) => Promise<AgentQuota>;
  removeAuth: (agentType: AgentProviderType) => Promise<void>;
}

export const useAgentAuthStore = create<AgentAuthState>((set, get) => ({
  accounts: [],
  isLoading: false,
  error: null,

  listAccounts: async () => {
    set({ isLoading: true, error: null });
    try {
      const accounts = await invoke<AgentAccountInfo[]>("list_agent_accounts");
      set({ accounts, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  startAuth: async (agentType: AgentProviderType) => {
    const start = await invoke<AgentAuthStart>("start_agent_auth", { agentType });
    try {
      await openUrl(start.authUrl);
    } catch (error) {
      console.warn("Failed to open browser for auth:", error);
    }
    return start;
  },

  completeAuth: async (flowId: string) => {
    const account = await invoke<AgentAccountInfo>("complete_agent_auth", { flowId });
    set((state) => {
      const has = state.accounts.some((a) => a.agentType === account.agentType);
      const accounts = has
        ? state.accounts.map((a) => (a.agentType === account.agentType ? account : a))
        : [...state.accounts, account];
      return { accounts };
    });
    return account;
  },

  getQuota: async (agentType: AgentProviderType) => {
    return invoke<AgentQuota>("get_agent_quota", { agentType });
  },

  removeAuth: async (agentType: AgentProviderType) => {
    await invoke("remove_agent_auth", { agentType });
    await get().listAccounts();
  },
}));
