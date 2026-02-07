import { useEffect, useMemo } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { ClaudeCodeConfig } from "@/components/agents/claude-code-config";
import { CodexConfig } from "@/components/agents/codex-config";
import { GeminiCLIConfig } from "@/components/agents/gemini-cli-config";
import { AgentConfig } from "@/components/agents/agent-config";
import { useAgents } from "@/hooks/use-agents";
import { getAgentName, getDefaultConfigPath } from "@/lib/agents";
import type { AgentType } from "@/types";

const VALID_AGENT_TYPES: AgentType[] = ["ClaudeCode", "Codex", "GeminiCLI", "Antigravity"];

function isValidAgentType(value: string | undefined): value is AgentType {
  return value != null && VALID_AGENT_TYPES.includes(value as AgentType);
}

export function AgentConfigPage() {
  const { agentType: paramAgentType } = useParams<{ agentType: string }>();
  const navigate = useNavigate();
  const { agents } = useAgents();

  const agentType = useMemo(() => {
    if (!isValidAgentType(paramAgentType)) return null;
    return paramAgentType;
  }, [paramAgentType]);

  const discoveredAgent = useMemo(() => {
    if (!agentType) return null;
    return agents.find((a) => a.agentType === agentType) ?? null;
  }, [agentType, agents]);

  const configPath = discoveredAgent?.configPath ?? null;
  const defaultConfigPath = agentType ? getDefaultConfigPath(agentType) : "";

  useEffect(() => {
    if (paramAgentType != null && !isValidAgentType(paramAgentType)) {
      navigate("/agents", { replace: true });
    }
  }, [paramAgentType, navigate]);

  if (agentType == null) {
    return null;
  }

  const title = `${getAgentName(agentType)} – Configuration`;
  const description = "Edit agent configuration. Changes may require an agent restart.";

  return (
    <MainContent title={title} description={description}>
      <div className="space-y-4">
        <Link
          to="/agents"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Coding Agents
        </Link>

        {agentType === "ClaudeCode" && (
          <ClaudeCodeConfig
            configPath={configPath}
            defaultConfigPath={defaultConfigPath}
          />
        )}
        {agentType === "Codex" && (
          <CodexConfig
            configPath={configPath}
            defaultConfigPath={defaultConfigPath}
          />
        )}
        {agentType === "GeminiCLI" && (
          <GeminiCLIConfig
            configPath={configPath}
            defaultConfigPath={defaultConfigPath}
          />
        )}
        {agentType === "Antigravity" && (
          <AgentConfig agentType={agentType} configPath={configPath} />
        )}
      </div>
    </MainContent>
  );
}
