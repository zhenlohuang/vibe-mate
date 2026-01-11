export type AgentType = "ClaudeCode" | "Codex" | "GeminiCLI";

export interface CodingAgent {
  agentType: AgentType;
  name: string;
  configPath: string | null;
}
