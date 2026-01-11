import type { AgentType } from "./agent";

export type Theme = "Dark" | "Light" | "System";

export interface AppConfig {
  enableProxy: boolean;
  proxyHost: string | null;
  proxyPort: number | null;
  noProxy: string[];
  appPort: number;
  theme: Theme;
  language: string;
  updatedAt: string;
}

export interface UpdateAppConfigInput {
  enableProxy?: boolean;
  proxyHost?: string | null;
  proxyPort?: number | null;
  noProxy?: string[];
  appPort?: number;
  theme?: Theme;
  language?: string;
}

export interface AgentConfigItem {
  type: AgentType;
  configFile?: string | null;
}

export interface UpdateAgentsConfigInput {
  agents?: AgentConfigItem[];
}

export interface LatencyResult {
  success: boolean;
  latencyMs: number | null;
  error: string | null;
}

export interface ProxyStatus {
  isRunning: boolean;
  port: number;
  requestCount: number;
}
