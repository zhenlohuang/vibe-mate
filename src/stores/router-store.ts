import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { RoutingRule, CreateRuleInput, UpdateRuleInput } from "@/types";

interface RouterState {
  rules: RoutingRule[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchRules: () => Promise<void>;
  createRule: (input: CreateRuleInput) => Promise<RoutingRule>;
  updateRule: (id: string, input: UpdateRuleInput) => Promise<RoutingRule>;
  deleteRule: (id: string) => Promise<void>;
  reorderRules: (ruleIds: string[]) => Promise<void>;
}

export const useRouterStore = create<RouterState>((set) => ({
  rules: [],
  isLoading: false,
  error: null,

  fetchRules: async () => {
    set({ isLoading: true, error: null });
    try {
      const rules = await invoke<RoutingRule[]>("list_rules");
      set({ rules, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  createRule: async (input: CreateRuleInput) => {
    try {
      const rule = await invoke<RoutingRule>("create_rule", { input });
      set((state) => ({
        rules: [...state.rules, rule],
      }));
      return rule;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateRule: async (id: string, input: UpdateRuleInput) => {
    try {
      const rule = await invoke<RoutingRule>("update_rule", { id, input });
      set((state) => ({
        rules: state.rules.map((r) => (r.id === id ? rule : r)),
      }));
      return rule;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteRule: async (id: string) => {
    try {
      await invoke("delete_rule", { id });
      set((state) => ({
        rules: state.rules.filter((r) => r.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  reorderRules: async (ruleIds: string[]) => {
    try {
      await invoke("reorder_rules", { ruleIds });
      // Reorder locally
      set((state) => {
        const priorityMap = new Map(
          ruleIds.map((id, index) => [id, index + 1])
        );
        const updatedRules = state.rules.map((rule) => {
          const nextPriority = priorityMap.get(rule.id);
          return nextPriority ? { ...rule, priority: nextPriority } : rule;
        });
        return { rules: updatedRules };
      });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
