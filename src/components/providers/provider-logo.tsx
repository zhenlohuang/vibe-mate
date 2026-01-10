import type { ProviderType } from "@/types";
import { cn } from "@/lib/utils";

interface ProviderLogoProps {
  type: ProviderType;
  className?: string;
}

export function ProviderLogo({ type, className }: ProviderLogoProps) {
  const logoMap: Record<ProviderType, { bg: string; text: string; label: string }> = {
    OpenAI: { bg: "bg-emerald-500/10", text: "text-emerald-400", label: "AI" },
    Anthropic: { bg: "bg-orange-500/10", text: "text-orange-400", label: "A" },
    Google: { bg: "bg-blue-500/10", text: "text-blue-400", label: "G" },
    Azure: { bg: "bg-sky-500/10", text: "text-sky-400", label: "Az" },
    Custom: { bg: "bg-purple-500/10", text: "text-purple-400", label: "C" },
  };

  const logo = logoMap[type];

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

