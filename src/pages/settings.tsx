import { useState, useEffect } from "react";
import { motion } from "motion/react";
import { Network, Save, Loader2 } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useAppConfig } from "@/hooks/use-tauri";
import { useToast } from "@/hooks/use-toast";
import type { UpdateAppConfigInput } from "@/types";

export function SettingsPage() {
  const { appConfig, updateConfig } = useAppConfig();
  const { toast } = useToast();
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const [formData, setFormData] = useState({
    enableProxy: false,
    proxyUrl: "",
    noProxy: "",
  });

  useEffect(() => {
    if (appConfig) {
      setFormData({
        enableProxy: appConfig.enableProxy || false,
        proxyUrl: appConfig.proxyUrl || "",
        noProxy: appConfig.noProxy?.join(", ") || "",
      });
    }
  }, [appConfig]);

  const handleFieldChange = (field: string, value: string | boolean) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setHasChanges(true);
  };

  const handleCancel = () => {
    if (appConfig) {
      setFormData({
        enableProxy: appConfig.enableProxy || false,
        proxyUrl: appConfig.proxyUrl || "",
        noProxy: appConfig.noProxy?.join(", ") || "",
      });
    }
    setHasChanges(false);
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const noProxyList = formData.noProxy
        .split(",")
        .map((item) => item.trim())
        .filter((item) => item.length > 0);

      const input: UpdateAppConfigInput = {
        enableProxy: formData.enableProxy,
        proxyUrl: formData.proxyUrl || null,
        noProxy: noProxyList,
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
      description="Upstream proxy and network settings."
    >
      <div className="max-w-xl space-y-6">
        {/* Network Settings */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="space-y-4"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
              <Network className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-base font-medium">Network Settings</h2>
          </div>

          <div className="grid grid-cols-[120px_1fr] items-center gap-x-4 gap-y-3 pl-8">
            <Label
              htmlFor="enableProxy"
              className="text-sm text-muted-foreground"
            >
              Enable Proxy
            </Label>
            <Switch
              id="enableProxy"
              checked={formData.enableProxy}
              onCheckedChange={(checked) =>
                handleFieldChange("enableProxy", checked)
              }
              className="scale-90"
            />

            {formData.enableProxy && (
              <>
                <Label
                  htmlFor="proxyUrl"
                  className="text-sm text-muted-foreground"
                >
                  Proxy Address
                </Label>
                <div className="space-y-1">
                  <Input
                    id="proxyUrl"
                    value={formData.proxyUrl}
                    onChange={(e) =>
                      handleFieldChange("proxyUrl", e.target.value)
                    }
                    placeholder="http://127.0.0.1:7890"
                    className="h-8 font-mono text-sm"
                  />
                  <p className="text-meta text-muted-foreground">
                    Supports http, https, and socks5 protocols (e.g. socks5://127.0.0.1:1080)
                  </p>
                </div>

                <Label
                  htmlFor="noProxy"
                  className="text-sm text-muted-foreground"
                >
                  No Proxy
                </Label>
                <div className="space-y-1">
                  <Input
                    id="noProxy"
                    value={formData.noProxy}
                    onChange={(e) =>
                      handleFieldChange("noProxy", e.target.value)
                    }
                    placeholder="localhost, 127.0.0.1, *.local"
                    className="h-8 font-mono text-sm"
                  />
                  <p className="text-meta text-muted-foreground">
                    Comma-separated list of hosts to bypass proxy
                  </p>
                </div>
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
            variant="outline"
            onClick={handleCancel}
            disabled={isSaving || !hasChanges}
          >
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
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
