import { useState, useEffect } from "react";
import { motion } from "motion/react";
import { Network, Save, Loader2, Settings2 } from "lucide-react";
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
    port: 12345,
    enableProxy: false,
    proxyUrl: "",
    noProxy: "",
  });

  useEffect(() => {
    if (appConfig) {
      setFormData({
        port: appConfig.port ?? 12345,
        enableProxy: appConfig.enableProxy || false,
        proxyUrl: appConfig.proxyUrl || "",
        noProxy: appConfig.noProxy?.join(", ") || "",
      });
    }
  }, [appConfig]);

  const handleFieldChange = (field: string, value: string | boolean | number) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setHasChanges(true);
  };

  const handleCancel = () => {
    if (appConfig) {
      setFormData({
        port: appConfig.port ?? 12345,
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
        port: formData.port,
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
      description="Vibe Mate and network settings."
    >
      <div className="max-w-xl space-y-10">
        {/* Vibe Mate */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
          className="space-y-6"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
              <Settings2 className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-base font-medium">General</h2>
          </div>

          <div className="grid grid-cols-[140px_1fr] items-start gap-x-6 gap-y-6 pl-8">
            <Label
              htmlFor="appPort"
              className="text-sm text-muted-foreground pt-2"
            >
              Proxy Port
            </Label>
            <div className="space-y-1.5">
              <Input
                id="appPort"
                type="number"
                min={1}
                max={65535}
                value={formData.port}
                onChange={(e) =>
                  handleFieldChange("port", parseInt(e.target.value, 10) || 12345)
                }
                className="h-9 font-mono text-sm w-28 [appearance:textfield] [-moz-appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
              />
              <p className="text-meta text-muted-foreground">
                Port for the built-in proxy (app.port). Restart proxy for changes to take effect.
              </p>
            </div>
          </div>
        </motion.div>

        {/* Network */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.05 }}
          className="space-y-6"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-primary/10">
              <Network className="h-3.5 w-3.5 text-primary" />
            </div>
            <h2 className="text-base font-medium">Network</h2>
          </div>

          <div className="grid grid-cols-[140px_1fr] items-start gap-x-6 gap-y-6 pl-8">
            <Label
              htmlFor="enableProxy"
              className="text-sm text-muted-foreground pt-2.5"
            >
              Enable Proxy
            </Label>
            <div className="flex items-center">
              <Switch
                id="enableProxy"
                checked={formData.enableProxy}
                onCheckedChange={(checked) =>
                  handleFieldChange("enableProxy", checked)
                }
                className="scale-90"
              />
            </div>

            {formData.enableProxy && (
              <>
                <Label
                  htmlFor="proxyUrl"
                  className="text-sm text-muted-foreground pt-2"
                >
                  Proxy Address
                </Label>
                <div className="space-y-1.5">
                  <Input
                    id="proxyUrl"
                    value={formData.proxyUrl}
                    onChange={(e) =>
                      handleFieldChange("proxyUrl", e.target.value)
                    }
                    placeholder="http://127.0.0.1:7890"
                    className="h-9 font-mono text-sm"
                  />
                  <p className="text-meta text-muted-foreground">
                    Supports http, https, and socks5 (e.g. socks5://127.0.0.1:1080)
                  </p>
                </div>

                <Label
                  htmlFor="noProxy"
                  className="text-sm text-muted-foreground pt-2"
                >
                  No Proxy
                </Label>
                <div className="space-y-1.5">
                  <Input
                    id="noProxy"
                    value={formData.noProxy}
                    onChange={(e) =>
                      handleFieldChange("noProxy", e.target.value)
                    }
                    placeholder="localhost, 127.0.0.1, *.local"
                    className="h-9 font-mono text-sm"
                  />
                  <p className="text-meta text-muted-foreground">
                    Comma-separated hosts to bypass proxy
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
          transition={{ duration: 0.3, delay: 0.1 }}
          className="flex items-center gap-3 pt-6 border-t border-border/40"
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
                <Save className="mr-1.5 h-3.5 w-3.5" />
                Save
              </>
            )}
          </Button>
        </motion.div>
      </div>
    </MainContent>
  );
}
