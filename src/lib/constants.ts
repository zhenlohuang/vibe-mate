import type { AgentType, AgentProviderType } from "@/types";

export const PROVIDER_TYPES = [
  { value: "OpenAI", label: "OpenAI", logo: "openai" },
  { value: "Anthropic", label: "Anthropic", logo: "anthropic" },
  { value: "Google", label: "Google", logo: "google" },
  { value: "OpenRouter", label: "OpenRouter", logo: "custom" },
  { value: "Custom", label: "Custom", logo: "custom" },
] as const;

export const AGENT_TYPES = [
  { value: "ClaudeCode", label: "Claude Code", logo: "anthropic" },
  { value: "Codex", label: "Codex", logo: "custom" },
  { value: "GeminiCli", label: "Gemini CLI", logo: "google" },
  { value: "Antigravity", label: "Antigravity", logo: "custom" },
] as const;

/** Map from agent detection type (AgentType) to auth/quota type (AgentProviderType) */
const AGENT_TYPE_TO_PROVIDER: Record<AgentType, AgentProviderType> = {
  ClaudeCode: "ClaudeCode",
  Codex: "Codex",
  GeminiCLI: "GeminiCli",
  Antigravity: "Antigravity",
};

export function agentTypeToProviderType(agentType: AgentType): AgentProviderType {
  return AGENT_TYPE_TO_PROVIDER[agentType];
}
