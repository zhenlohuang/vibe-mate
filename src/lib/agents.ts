import type { AgentType } from "@/types";

export const AGENT_CATALOG: Record<
  AgentType,
  { name: string; defaultConfigPath: string }
> = {
  ClaudeCode: {
    name: "Claude Code",
    defaultConfigPath: "~/.claude/settings.json",
  },
  Codex: {
    name: "Codex",
    defaultConfigPath: "~/.codex/config.toml",
  },
  GeminiCLI: {
    name: "Gemini CLI",
    defaultConfigPath: "~/.gemini/settings.json",
  },
  Antigravity: {
    name: "Antigravity",
    defaultConfigPath: "~/.antigravity/settings.json",
  },
};

export const AGENT_TYPES = Object.keys(AGENT_CATALOG) as AgentType[];

export function getAgentName(agentType: AgentType) {
  return AGENT_CATALOG[agentType].name;
}

export function getDefaultConfigPath(agentType: AgentType) {
  return AGENT_CATALOG[agentType].defaultConfigPath;
}
