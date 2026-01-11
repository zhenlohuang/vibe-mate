export const PROVIDER_TYPES = [
  { value: "OpenAI", label: "OpenAI", logo: "openai" },
  { value: "Anthropic", label: "Anthropic", logo: "anthropic" },
  { value: "Google", label: "Google", logo: "google" },
  { value: "Azure", label: "Azure", logo: "azure" },
  { value: "Custom", label: "Custom", logo: "custom" },
] as const;

export const PROXY_TYPES = [
  { value: "SOCKS5", label: "SOCKS5" },
  { value: "HTTP", label: "HTTP" },
  { value: "HTTPS", label: "HTTPS" },
] as const;

export const PROXY_MODES = [
  { value: "System", label: "System Proxy" },
  { value: "Custom", label: "Custom Proxy" },
  { value: "None", label: "No Proxy" },
] as const;
