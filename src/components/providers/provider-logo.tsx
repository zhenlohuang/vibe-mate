import type { AgentProviderType, ProviderType } from "@/types";
import { cn } from "@/lib/utils";

type LogoType = ProviderType | AgentProviderType;

interface ProviderLogoProps {
  type: LogoType;
  className?: string;
}

export function ProviderLogo({ type, className }: ProviderLogoProps) {
  const logoMap: Record<LogoType, { bg: string; text: string; label: string }> = {
    // Model Providers
    OpenAI: { bg: "bg-emerald-500/10", text: "text-emerald-400", label: "AI" },
    Anthropic: { bg: "bg-orange-500/10", text: "text-orange-400", label: "A" },
    Google: { bg: "bg-blue-500/10", text: "text-blue-400", label: "G" },
    OpenRouter: { bg: "bg-violet-500/10", text: "text-violet-400", label: "OR" },
    Custom: { bg: "bg-purple-500/10", text: "text-purple-400", label: "C" },
    // Agent Providers
    ClaudeCode: { bg: "bg-orange-500/10", text: "text-orange-400", label: "CC" },
    Codex: { bg: "bg-teal-500/10", text: "text-teal-400", label: "CX" },
    GeminiCli: { bg: "bg-blue-500/10", text: "text-blue-400", label: "GC" },
    Antigravity: { bg: "bg-pink-500/10", text: "text-pink-400", label: "AG" },
  };

  const logo = logoMap[type] || { bg: "bg-gray-500/10", text: "text-gray-400", label: "?" };

  return (
    <div
      className={cn(
        "flex h-7 w-7 items-center justify-center rounded-md text-xs font-bold shrink-0",
        logo.bg,
        logo.text,
        className
      )}
    >
      {logo.label}
    </div>
  );
}

