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
import { PROXY_TYPES } from "@/lib/constants";
import type { ProxyType, UpdateAppConfigInput } from "@/types";

export function SettingsPage() {
  const { appConfig, updateConfig } = useAppConfig();
  const { toast } = useToast();
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const [formData, setFormData] = useState({
    proxyServerPort: "12345",
    proxyType: "SOCKS5" as ProxyType,
    proxyHost: "127.0.0.1",
    proxyPort: "7890",
  });

  useEffect(() => {
    if (appConfig) {
      setFormData({
        proxyServerPort: appConfig.proxyServerPort?.toString() || "12345",
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
      description="Manage your local vibe ports and upstream proxy connections to ensure seamless agent communication."
    >
      <div className="space-y-4">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="rounded-lg border border-dashed border-border/60 p-4"
        >
          <div className="flex items-center gap-2 mb-4">
            <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary/10">
              <SettingsIcon className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-sm font-semibold">App Settings</h2>
          </div>

          <div className="space-y-3">
            <div className="space-y-1.5">
              <Label htmlFor="port">Port</Label>
              <div className="relative">
                <Input
                  id="port"
                  type="text"
                  value={formData.proxyServerPort}
                  onChange={(e) => handleFieldChange("proxyServerPort", e.target.value)}
                  placeholder="12345"
                  className="font-mono pr-6"
                />
                {formData.proxyServerPort && (
                  <span className="absolute right-2 top-1/2 -translate-y-1/2 h-1.5 w-1.5 rounded-full bg-success" />
                )}
              </div>
              <p className="text-[10px] text-muted-foreground">
                The local port where the Vibe Mate interface is served.
              </p>
            </div>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.1 }}
          className="rounded-lg border border-dashed border-border/60 p-4"
        >
          <div className="flex items-center gap-2 mb-4">
            <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary/10">
              <Network className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-sm font-semibold">Network Settings</h2>
          </div>

          <div className="grid gap-4 sm:grid-cols-3">
            <div className="space-y-1.5">
              <Label htmlFor="proxyType">Proxy Type</Label>
              <Select
                value={formData.proxyType}
                onValueChange={(value) => handleFieldChange("proxyType", value)}
              >
                <SelectTrigger id="proxyType">
                  <SelectValue placeholder="Select proxy type" />
                </SelectTrigger>
                <SelectContent>
                  {PROXY_TYPES.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="proxyHost">Host</Label>
              <Input
                id="proxyHost"
                value={formData.proxyHost}
                onChange={(e) => handleFieldChange("proxyHost", e.target.value)}
                placeholder="127.0.0.1"
                className="font-mono"
              />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="proxyPort">Port</Label>
              <Input
                id="proxyPort"
                type="text"
                value={formData.proxyPort}
                onChange={(e) => handleFieldChange("proxyPort", e.target.value)}
                placeholder="7890"
                className="font-mono"
              />
            </div>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.2 }}
          className="flex items-center justify-end gap-2 pt-3 border-t border-border/40"
        >
          <Button
            variant="ghost"
            onClick={handleCancel}
            disabled={isSaving || !hasChanges}
          >
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving}
            className="min-w-[100px]"
          >
            {isSaving ? (
              <>
                <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="mr-1.5 h-3.5 w-3.5" />
                Save Changes
              </>
            )}
          </Button>
        </motion.div>
      </div>
    </MainContent>
  );
}

