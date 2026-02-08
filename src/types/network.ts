export interface AppConfig {
  /** Proxy server listen port (config key: app.port) */
  port: number;
  enableProxy: boolean;
  proxyUrl: string | null;
  noProxy: string[];
  updatedAt: string;
}

export interface UpdateAppConfigInput {
  port?: number;
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
