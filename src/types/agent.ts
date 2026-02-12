export type AgentType = "ClaudeCode" | "Codex" | "GeminiCLI" | "Antigravity";

export type AgentStatus =
  | "Installed"
  | "NotInstalled"
  | "Authenticated"
  | "NotAuthenticated";

export interface CodingAgent {
  agentType: AgentType;
  name: string;
  version?: string | null;
  status: AgentStatus;
  executablePath?: string | null;
  configPath: string | null;
  authPath?: string | null;
  /** Whether to show this agent on the Dashboard. Default true when new or missing. */
  featured?: boolean;
  /** Whether VibeMate proxy auto-config is enabled for this agent. */
  proxyEnabled?: boolean;
}
