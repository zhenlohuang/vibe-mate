export type ProviderType = "OpenAI" | "Anthropic" | "Google" | "Azure" | "Custom";

export type ProviderStatus = "Connected" | "Disconnected" | "Error";

export interface Provider {
  id: string;
  name: string;
  type: ProviderType;
  apiBaseUrl: string;
  apiKey: string;
  isDefault: boolean;
  status: ProviderStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CreateProviderInput {
  name: string;
  type: ProviderType;
  apiBaseUrl: string;
  apiKey: string;
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
