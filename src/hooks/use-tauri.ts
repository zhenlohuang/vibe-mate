import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "@/stores/app-store";
import type { ProxyStatus, AppConfig, UpdateAppConfigInput, LatencyResult } from "@/types";

// Hook for proxy status
export function useProxyStatus() {
  const { proxyStatus, setProxyStatus } = useAppStore();

  const fetchStatus = useCallback(async () => {
    try {
      // The backend now does the actual health check
      const status = await invoke<ProxyStatus>("proxy_status");
      setProxyStatus(status);
    } catch (error) {
      console.error("Failed to fetch proxy status:", error);
    }
  }, [setProxyStatus]);

  const startProxy = useCallback(async () => {
    try {
      await invoke("start_proxy");
      // Wait a bit for the server to start, then check status
      await new Promise(resolve => setTimeout(resolve, 1000));
      await fetchStatus();
    } catch (error) {
      console.error("Failed to start proxy:", error);
      throw error;
    }
  }, [fetchStatus]);

  const stopProxy = useCallback(async () => {
    try {
      await invoke("stop_proxy");
      await fetchStatus();
    } catch (error) {
      console.error("Failed to stop proxy:", error);
      throw error;
    }
  }, [fetchStatus]);

  useEffect(() => {
    fetchStatus();
    // Poll for status updates every 5 seconds
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  return { proxyStatus, startProxy, stopProxy, refetch: fetchStatus };
}

// Hook for app config
export function useAppConfig() {
  const { appConfig, setAppConfig } = useAppStore();

  const fetchConfig = useCallback(async () => {
    try {
      const config = await invoke<AppConfig>("get_config");
      setAppConfig(config);
      return config;
    } catch (error) {
      console.error("Failed to fetch config:", error);
      throw error;
    }
  }, [setAppConfig]);

  const updateConfig = useCallback(
    async (input: UpdateAppConfigInput) => {
      try {
        const config = await invoke<AppConfig>("update_config", { input });
        setAppConfig(config);
        return config;
      } catch (error) {
        console.error("Failed to update config:", error);
        throw error;
      }
    },
    [setAppConfig]
  );

  const testLatency = useCallback(async (): Promise<LatencyResult> => {
    try {
      const result = await invoke<LatencyResult>("test_latency");
      return result;
    } catch (error) {
      console.error("Failed to test latency:", error);
      return { success: false, latencyMs: null, error: String(error) };
    }
  }, []);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  return { appConfig, updateConfig, testLatency, refetch: fetchConfig };
}

// Hook for system info
export function useSystemInfo() {
  const getVersion = useCallback(async () => {
    try {
      const version = await invoke<string>("get_version");
      return version;
    } catch (error) {
      console.error("Failed to get version:", error);
      return "0.1.0";
    }
  }, []);

  return { getVersion };
}

