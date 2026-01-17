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
  authPath?: string;
  isDefault: boolean;
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
  authPath?: string;
}

export interface UpdateProviderInput {
  name?: string;
  apiBaseUrl?: string;
  apiKey?: string;
  authPath?: string;
}

export interface ConnectionStatus {
  isConnected: boolean;
  latencyMs?: number;
  error?: string;
}
