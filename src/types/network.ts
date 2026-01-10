export type ProxyMode = "System" | "Custom" | "None";

export type ProxyType = "SOCKS5" | "HTTP" | "HTTPS";

export type Theme = "Dark" | "Light" | "System";

export interface AppConfig {
  proxyMode: ProxyMode;
  proxyType: ProxyType;
  proxyHost: string | null;
  proxyPort: number | null;
  proxyServerPort: number;
  theme: Theme;
  language: string;
  updatedAt: string;
}

export interface UpdateAppConfigInput {
  proxyMode?: ProxyMode;
  proxyType?: ProxyType;
  proxyHost?: string | null;
  proxyPort?: number | null;
  proxyServerPort?: number;
  theme?: Theme;
  language?: string;
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

