import { motion } from "motion/react";
import { FileCode, Save, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@/hooks/use-toast";

interface CodexConfigProps {
  configPath: string | null;
  defaultConfigPath: string;
}

export function CodexConfig({
  configPath,
  defaultConfigPath,
}: CodexConfigProps) {
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const { toast } = useToast();
  const resolvedConfigPath = configPath || defaultConfigPath;

  // Load config on mount and when path changes
  useEffect(() => {
    loadConfig();
  }, [configPath]);

  const loadConfig = async () => {
    setIsLoading(true);
    try {
      const config = await invoke<string>("read_agent_config", {
        agentType: "Codex",
        configPath: configPath || undefined,
      });
      setContent(config);
    } catch (error) {
      toast({
        title: "Failed to load config",
        description: String(error),
        variant: "destructive",
      });
      // Set default config if file doesn't exist (TOML format)
      setContent(`[agent]
name = "codex"
version = "2.1.0"

[settings]
max_concurrent_tasks = 5
temperature = 0.3
default_model = "gpt-4-turbo"
auto_approval = true
context_window_optimization = false
safety_filter = "moderate"
local_cache = true
completion_timeout = 30
`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await invoke("save_agent_config", { 
        agentType: "Codex", 
        content,
        configPath: configPath || undefined,
      });
      toast({
        title: "Config Saved",
        description: "Codex configuration has been saved successfully.",
      });
    } catch (error) {
      toast({
        title: "Failed to save config",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: "auto" }}
      exit={{ opacity: 0, height: 0 }}
      className="border-t border-border bg-muted/30"
    >
      <div className="p-4 space-y-3">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <FileCode className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="text-sm font-medium">Codex Configuration</span>
          </div>
          <div className="text-meta text-muted-foreground font-mono">
            <span>{resolvedConfigPath}</span>
          </div>
        </div>

        {/* Config Content */}
        <div className="relative">
          {isLoading ? (
            <div className="flex items-center justify-center p-8 bg-background border border-border rounded-md">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : (
            <Textarea
              value={content}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setContent(e.target.value)}
              className="font-mono text-meta min-h-[200px] bg-background border-border"
              placeholder="Enter Codex configuration TOML..."
            />
          )}
        </div>

        {/* Footer with note */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-meta text-muted-foreground">
            <div className="h-1 w-1 rounded-full bg-muted-foreground/50" />
            <span className="italic">Changes require Codex restart to take effect.</span>
          </div>
          <Button 
            size="sm" 
            className="h-8 text-sm gap-1.5" 
            onClick={handleSave}
            disabled={isSaving || isLoading}
          >
            {isSaving ? (
              <>
                <Loader2 className="h-3 w-3 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="h-3 w-3" />
                Save Configuration
              </>
            )}
          </Button>
        </div>
      </div>
    </motion.div>
  );
}
