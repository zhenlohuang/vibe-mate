import { Terminal } from "lucide-react";
import { cn } from "@/lib/utils";

interface LogEntry {
  timestamp: string;
  type: "info" | "success" | "error" | "warning" | "agent";
  agent?: string;
  message: string;
}

const mockLogs: LogEntry[] = [
  { timestamp: "14:02:22", type: "info", message: "System initialized. Watching 3 agents..." },
  { timestamp: "14:02:23", type: "success", message: "Connecting to proxy server ws://localhost:12345... OK" },
  { timestamp: "14:05:10", type: "agent", agent: "Claude Code", message: "requesting file access: /src/components/Dashboard.tsx" },
  { timestamp: "14:05:11", type: "info", message: "Parsing Abstract Syntax Tree... Done (45ms)" },
  { timestamp: "14:05:12", type: "info", message: "Generated 3 potential refactors. Analyzing complexity..." },
  { timestamp: "14:08:45", type: "agent", agent: "Gemini CLI", message: "execution started: `npm run build`" },
  { timestamp: "14:08:46", type: "info", message: "> next build    > Creating an optimized production build..." },
  { timestamp: "14:12:01", type: "error", message: "Error: Connection lost to AutoGPT Legacy agent (TIMEOUT)" },
  { timestamp: "14:12:02", type: "info", message: "Retrying connection (1/3)..." },
  { timestamp: "14:15:33", type: "warning", message: "Waiting for next command" },
];

function getLogColor(type: LogEntry["type"]) {
  switch (type) {
    case "success":
      return "text-success";
    case "error":
      return "text-destructive";
    case "warning":
      return "text-warning";
    case "agent":
      return "text-accent";
    default:
      return "text-muted-foreground";
  }
}

export function TerminalOutput() {
  return (
    <div className="rounded-lg border border-border bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-3 py-2 bg-secondary/30">
        <div className="flex items-center gap-1.5">
          <Terminal className="h-3.5 w-3.5 text-muted-foreground" />
          <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            Terminal Output
          </span>
        </div>
        <div className="flex items-center gap-1">
          <div className="h-2 w-2 rounded-full bg-destructive/80" />
          <div className="h-2 w-2 rounded-full bg-warning/80" />
          <div className="h-2 w-2 rounded-full bg-success/80" />
        </div>
      </div>

      {/* Terminal Content */}
      <div className="h-[200px] overflow-auto p-3 font-mono text-[11px] bg-background/50">
        {mockLogs.map((log, index) => (
          <div key={index} className="flex gap-1.5 leading-relaxed">
            <span className="text-muted-foreground/60 flex-shrink-0">
              [{log.timestamp}]
            </span>
            {log.agent && (
              <span className={cn("font-semibold", getLogColor(log.type))}>
                {log.agent}
              </span>
            )}
            <span
              className={cn(
                log.type === "error" && "text-destructive",
                log.type === "success" && log.message.includes("OK") && "text-success",
                log.type === "warning" && "text-warning",
                log.type === "info" && "text-muted-foreground"
              )}
            >
              {log.message}
            </span>
            {log.type === "warning" && (
              <span className="inline-block w-1.5 h-3 bg-foreground animate-pulse" />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

