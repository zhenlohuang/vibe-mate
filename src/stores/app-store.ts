import { create } from "zustand";
import type { ProxyStatus, AppConfig } from "@/types";

interface AppState {
  // Proxy status
  proxyStatus: ProxyStatus;
  setProxyStatus: (status: ProxyStatus) => void;

  // App config
  appConfig: AppConfig | null;
  setAppConfig: (config: AppConfig) => void;

  // Loading states
  isLoading: boolean;
  setLoading: (loading: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Initial proxy status
  proxyStatus: {
    isRunning: false,
    port: 12345,
    requestCount: 0,
  },
  setProxyStatus: (status) => set({ proxyStatus: status }),

  // App config
  appConfig: null,
  setAppConfig: (config) => set({ appConfig: config }),

  // Loading
  isLoading: false,
  setLoading: (loading) => set({ isLoading: loading }),
}));

