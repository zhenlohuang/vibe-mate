export interface AppConfig {
  enableProxy: boolean;
  proxyUrl: string | null;
  noProxy: string[];
  updatedAt: string;
}

export interface UpdateAppConfigInput {
  enableProxy?: boolean;
  proxyUrl?: string | null;
  noProxy?: string[];
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
