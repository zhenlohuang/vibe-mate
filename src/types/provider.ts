export type ProviderCategory = "Model" | "Agent";

export type ModelProviderType = "OpenAI" | "Anthropic" | "Google" | "OpenRouter" | "Custom";

export type AgentProviderType = "Codex" | "ClaudeCode" | "GeminiCli" | "Antigravity";

export type ProviderType = ModelProviderType | AgentProviderType;

export type ProviderStatus = "Connected" | "Disconnected" | "Error";

export interface Provider {
  id: string;
  name: string;
  category: ProviderCategory;
  type: ProviderType;
  apiBaseUrl?: string;
  apiKey?: string;
  status: ProviderStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CreateProviderInput {
  name: string;
  category: ProviderCategory;
  type: ProviderType;
  apiBaseUrl?: string;
  apiKey?: string;
}

export interface UpdateProviderInput {
  name?: string;
  apiBaseUrl?: string;
  apiKey?: string;
}

export interface ConnectionStatus {
  isConnected: boolean;
  latencyMs?: number;
  error?: string;
}

export interface AgentAuthStart {
  flowId: string;
  authUrl: string;
}

export interface AgentQuota {
  planType?: string | null;
  limitReached?: boolean | null;
  sessionUsedPercent: number;
  sessionResetAt?: number | null;
  weekUsedPercent: number;
  weekResetAt?: number | null;
  entries?: AgentQuotaEntry[] | null;
  note?: string | null;
}

export interface AgentQuotaEntry {
  label: string;
  usedPercent: number;
  resetAt?: number | null;
}
