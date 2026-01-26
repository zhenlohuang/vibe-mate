export interface AppConfig {
  enableProxy: boolean;
  proxyHost: string | null;
  proxyPort: number | null;
  noProxy: string[];
  updatedAt: string;
}

export interface UpdateAppConfigInput {
  enableProxy?: boolean;
  proxyHost?: string | null;
  proxyPort?: number | null;
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
