import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { CodingAgent, AgentType } from "@/types";

interface AgentState {
  agents: CodingAgent[];
  isLoading: boolean;
  error: string | null;

  // Actions
  discoverAgents: () => Promise<void>;
  checkStatus: (agentType: AgentType) => Promise<CodingAgent>;
  openLogin: (agentType: AgentType) => Promise<void>;
  getVersion: (agentType: AgentType) => Promise<string | null>;
}

export const useAgentStore = create<AgentState>((set) => ({
  agents: [],
  isLoading: false,
  error: null,

  discoverAgents: async () => {
    set({ isLoading: true, error: null });
    try {
      const agents = await invoke<CodingAgent[]>("discover_agents");
      set({ agents, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  checkStatus: async (agentType: AgentType) => {
    try {
      const agent = await invoke<CodingAgent>("check_status", { agentType });
      set((state) => ({
        agents: state.agents.map((a) =>
          a.agentType === agentType ? agent : a
        ),
      }));
      return agent;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  openLogin: async (agentType: AgentType) => {
    try {
      await invoke("open_login", { agentType });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getVersion: async (agentType: AgentType) => {
    try {
      const version = await invoke<string | null>("get_version", { agentType });
      return version;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));

