import { useState, useEffect } from "react";
import { motion } from "motion/react";
import { Settings as SettingsIcon, Network, Save, Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useAppConfig } from "@/hooks/use-tauri";
import { useToast } from "@/hooks/use-toast";
import { PROXY_TYPES, PROXY_MODES } from "@/lib/constants";
import type { ProxyType, ProxyMode, UpdateAppConfigInput } from "@/types";

export function SettingsPage() {
  const { appConfig, updateConfig } = useAppConfig();
  const { toast } = useToast();
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const [formData, setFormData] = useState({
    proxyServerPort: "12345",
    proxyMode: "System" as ProxyMode,
    proxyType: "SOCKS5" as ProxyType,
    proxyHost: "127.0.0.1",
    proxyPort: "7890",
  });

  useEffect(() => {
    if (appConfig) {
      setFormData({
        proxyServerPort: appConfig.proxyServerPort?.toString() || "12345",
        proxyMode: appConfig.proxyMode || "System",
        proxyType: appConfig.proxyType || "SOCKS5",
        proxyHost: appConfig.proxyHost || "127.0.0.1",
        proxyPort: appConfig.proxyPort?.toString() || "7890",
      });
    }
  }, [appConfig]);

  const handleFieldChange = (field: string, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setHasChanges(true);
  };

  const handleCancel = () => {
    if (appConfig) {
      setFormData({
        proxyServerPort: appConfig.proxyServerPort?.toString() || "12345",
        proxyMode: appConfig.proxyMode || "System",
        proxyType: appConfig.proxyType || "SOCKS5",
        proxyHost: appConfig.proxyHost || "127.0.0.1",
        proxyPort: appConfig.proxyPort?.toString() || "7890",
      });
    }
    setHasChanges(false);
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const input: UpdateAppConfigInput = {
        proxyServerPort: parseInt(formData.proxyServerPort) || 12345,
        proxyMode: formData.proxyMode,
        proxyType: formData.proxyType,
        proxyHost: formData.proxyHost || null,
        proxyPort: formData.proxyPort ? parseInt(formData.proxyPort) : null,
      };
      await updateConfig(input);
      toast({
        title: "Settings Saved",
        description: "Your settings have been updated successfully.",
      });
      setHasChanges(false);
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <MainContent
      title="Preferences"
      description="Manage your local vibe ports and upstream proxy connections."
    >
      <div className="max-w-xl space-y-6">
        {/* App Settings */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="space-y-4"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
              <SettingsIcon className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-sm font-medium">App Settings</h2>
          </div>

          <div className="grid grid-cols-[120px_1fr] items-center gap-x-4 gap-y-3 pl-8">
            <Label htmlFor="port" className="text-xs text-muted-foreground">
              Server Port
            </Label>
            <Input
              id="port"
              type="text"
              value={formData.proxyServerPort}
              onChange={(e) =>
                handleFieldChange("proxyServerPort", e.target.value)
              }
              placeholder="12345"
              className="h-8 w-32 font-mono text-sm"
            />
          </div>
        </motion.div>

        {/* Network Settings */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.1 }}
          className="space-y-4"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
              <Network className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-sm font-medium">Network Settings</h2>
          </div>

          <div className="grid grid-cols-[120px_1fr] items-center gap-x-4 gap-y-3 pl-8">
            <Label
              htmlFor="proxyMode"
              className="text-xs text-muted-foreground"
            >
              Proxy Mode
            </Label>
            <Select
              value={formData.proxyMode}
              onValueChange={(value) => handleFieldChange("proxyMode", value)}
            >
              <SelectTrigger id="proxyMode" className="h-8 w-40 text-sm">
                <SelectValue placeholder="Select mode" />
              </SelectTrigger>
              <SelectContent>
                {PROXY_MODES.map((mode) => (
                  <SelectItem key={mode.value} value={mode.value}>
                    {mode.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            {formData.proxyMode === "Custom" && (
              <>
                <Label
                  htmlFor="proxyType"
                  className="text-xs text-muted-foreground"
                >
                  Proxy Type
                </Label>
                <Select
                  value={formData.proxyType}
                  onValueChange={(value) =>
                    handleFieldChange("proxyType", value)
                  }
                >
                  <SelectTrigger id="proxyType" className="h-8 w-32 text-sm">
                    <SelectValue placeholder="Type" />
                  </SelectTrigger>
                  <SelectContent>
                    {PROXY_TYPES.map((type) => (
                      <SelectItem key={type.value} value={type.value}>
                        {type.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Label
                  htmlFor="proxyHost"
                  className="text-xs text-muted-foreground"
                >
                  Host
                </Label>
                <Input
                  id="proxyHost"
                  value={formData.proxyHost}
                  onChange={(e) =>
                    handleFieldChange("proxyHost", e.target.value)
                  }
                  placeholder="127.0.0.1"
                  className="h-8 w-48 font-mono text-sm"
                />

                <Label
                  htmlFor="proxyPort"
                  className="text-xs text-muted-foreground"
                >
                  Port
                </Label>
                <Input
                  id="proxyPort"
                  type="text"
                  value={formData.proxyPort}
                  onChange={(e) =>
                    handleFieldChange("proxyPort", e.target.value)
                  }
                  placeholder="7890"
                  className="h-8 w-24 font-mono text-sm"
                />
              </>
            )}
          </div>
        </motion.div>

        {/* Actions */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.2 }}
          className="flex items-center gap-2 pt-4 border-t border-border/40"
        >
          <Button
            variant="ghost"
            size="sm"
            onClick={handleCancel}
            disabled={isSaving || !hasChanges}
          >
            Cancel
          </Button>
          <Button size="sm" onClick={handleSave} disabled={isSaving}>
            {isSaving ? (
              <>
                <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="h-3.5 w-3.5" />
                Save
              </>
            )}
          </Button>
        </motion.div>
      </div>
    </MainContent>
  );
}
