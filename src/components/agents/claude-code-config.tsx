import { motion } from "motion/react";
import { FileCode, Save, Loader2, PencilLine, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@/hooks/use-toast";

interface ClaudeCodeConfigProps {
  configPath: string | null;
  defaultConfigPath: string;
  onUpdateConfigPath: (configPath: string) => Promise<void>;
}

export function ClaudeCodeConfig({
  configPath,
  defaultConfigPath,
  onUpdateConfigPath,
}: ClaudeCodeConfigProps) {
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isEditingPath, setIsEditingPath] = useState(false);
  const [pathValue, setPathValue] = useState(configPath || defaultConfigPath);
  const [isSavingPath, setIsSavingPath] = useState(false);
  const { toast } = useToast();
  const resolvedConfigPath = configPath || defaultConfigPath;

  // Load config on mount and when path changes
  useEffect(() => {
    loadConfig();
  }, [configPath]);

  useEffect(() => {
    if (!isEditingPath) {
      setPathValue(resolvedConfigPath);
    }
  }, [isEditingPath, resolvedConfigPath]);

  const loadConfig = async (overridePath?: string | null) => {
    setIsLoading(true);
    try {
      const configPathOverride = overridePath ?? configPath;
      const config = await invoke<string>("read_agent_config", { 
        agentType: "ClaudeCode",
        configPath: configPathOverride || undefined,
      });
      setContent(config);
    } catch (error) {
      toast({
        title: "Failed to load config",
        description: String(error),
        variant: "destructive",
      });
      // Set default config if file doesn't exist
      setContent(`{
  "agent": "claude-code",
  "version": "1.0.4",
  "settings": {
    "max_concurrent_tasks": 3,
    "temperature": 0.2,
    "default_model": "claude-3-5-sonnet",
    "auto_approval": false,
    "context_window_optimization": true,
    "safety_filter": "strict",
    "local_cache": true
  }
}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSavePath = async () => {
    const nextPath = pathValue.trim();
    setIsSavingPath(true);
    try {
      await onUpdateConfigPath(nextPath);
      toast({
        title: "Path Updated",
        description: "Claude Code config path has been updated.",
      });
      setIsEditingPath(false);
      await loadConfig(nextPath || null);
    } catch (error) {
      toast({
        title: "Failed to update path",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsSavingPath(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await invoke("save_agent_config", { 
        agentType: "ClaudeCode", 
        content,
        configPath: configPath || undefined,
      });
      toast({
        title: "Config Saved",
        description: "Claude Code configuration has been saved successfully.",
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
            <span className="text-xs font-medium">Claude Code Configuration</span>
          </div>
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground font-mono">
            {isEditingPath ? (
              <>
                <Input
                  value={pathValue}
                  onChange={(e) => setPathValue(e.target.value)}
                  className="h-6 text-[10px] font-mono"
                />
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={handleSavePath}
                  disabled={isSavingPath}
                >
                  <Check className="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => {
                    setIsEditingPath(false);
                    setPathValue(resolvedConfigPath);
                  }}
                  disabled={isSavingPath}
                >
                  <X className="h-3.5 w-3.5" />
                </Button>
              </>
            ) : (
              <>
                <span>{resolvedConfigPath}</span>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setIsEditingPath(true)}
                >
                  <PencilLine className="h-3.5 w-3.5" />
                </Button>
              </>
            )}
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
              className="font-mono text-[10px] min-h-[200px] bg-background border-border"
              placeholder="Enter Claude Code configuration JSON..."
            />
          )}
        </div>

        {/* Footer with note */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
            <div className="h-1 w-1 rounded-full bg-muted-foreground/50" />
            <span className="italic">Changes require Claude Code restart to take effect.</span>
          </div>
          <Button 
            size="sm" 
            className="h-7 text-xs gap-1.5" 
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
