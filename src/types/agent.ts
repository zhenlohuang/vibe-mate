export type AgentType = "ClaudeCode" | "GeminiCLI";

export type AgentStatus = "Installed" | "NotInstalled" | "Authenticated" | "NotAuthenticated";

export interface CodingAgent {
  agentType: AgentType;
  name: string;
  version: string | null;
  status: AgentStatus;
  executablePath: string | null;
  configPath: string | null;
}

