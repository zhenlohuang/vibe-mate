import { useState, useEffect } from "react";
import type { Provider, CreateProviderInput, ProviderType } from "@/types";
import { PROVIDER_TYPES } from "@/lib/constants";
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface ProviderFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  provider?: Provider;
  onSubmit: (data: CreateProviderInput) => Promise<void>;
  onDelete?: (id: string) => Promise<void>;
}

export function ProviderForm({
  open,
  onOpenChange,
  provider,
  onSubmit,
  onDelete,
}: ProviderFormProps) {
  const isEdit = !!provider;

  const [formData, setFormData] = useState<CreateProviderInput>({
    name: "",
    type: "OpenAI",
    apiBaseUrl: "",
    apiKey: "",
  });

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // Update form data when provider changes (for edit mode)
  useEffect(() => {
    if (provider) {
      setFormData({
        name: provider.name,
        type: provider.type,
        apiBaseUrl: provider.apiBaseUrl || "",
        apiKey: "", // Don't populate API key for security
      });
    } else {
      setFormData({
        name: "",
        type: "OpenAI",
        apiBaseUrl: "",
        apiKey: "",
      });
    }
  }, [provider]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      await onSubmit(formData);
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to submit provider:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!provider || !onDelete) return;
    setIsDeleting(true);
    try {
      await onDelete(provider.id);
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to delete provider:", error);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleTypeChange = (type: ProviderType) => {
    const defaultUrls: Record<ProviderType, string> = {
      OpenAI: "https://api.openai.com",
      Anthropic: "https://api.anthropic.com",
      Google: "https://generativelanguage.googleapis.com",
      OpenRouter: "https://openrouter.ai/api",
      Custom: "",
    };

    setFormData((prev) => ({
      ...prev,
      type,
      apiBaseUrl: prev.apiBaseUrl || defaultUrls[type],
      name: prev.name || type,
    }));
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle className="text-sm">
            {isEdit ? "Edit Provider" : "Add Provider"}
          </DialogTitle>
          <DialogDescription className="text-xs">
            {isEdit
              ? "Update the provider configuration."
              : "Add a new AI model provider to your configuration."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-3">
          {/* Provider Type */}
          <div className="space-y-1.5">
            <Label htmlFor="type">Provider Type</Label>
            <Select
              value={formData.type}
              onValueChange={(value) =>
                handleTypeChange(value as ProviderType)
              }
              disabled={isEdit}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select provider type" />
              </SelectTrigger>
              <SelectContent>
                {PROVIDER_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Name */}
          <div className="space-y-1.5">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={formData.name}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, name: e.target.value }))
              }
              placeholder="Enter a name for this provider"
              required
            />
          </div>

          {/* API Base URL */}
          <div className="space-y-1.5">
            <Label htmlFor="apiBaseUrl">API Base URL</Label>
            <Input
              id="apiBaseUrl"
              value={formData.apiBaseUrl}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, apiBaseUrl: e.target.value }))
              }
              placeholder="https://api.example.com"
              className="font-mono"
              required
            />
          </div>

          {/* API Key */}
          <div className="space-y-1.5">
            <Label htmlFor="apiKey">API Key</Label>
            <Input
              id="apiKey"
              type="password"
              value={formData.apiKey}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, apiKey: e.target.value }))
              }
              placeholder="sk-..."
              className="font-mono"
              required={!isEdit}
            />
            {isEdit && (
              <p className="text-[10px] text-muted-foreground">
                Leave empty to keep the existing API key
              </p>
            )}
          </div>

          <DialogFooter className="flex-row justify-between sm:justify-between">
            {isEdit && onDelete ? (
              <Button
                type="button"
                variant="destructive"
                onClick={handleDelete}
                disabled={isDeleting || isSubmitting}
              >
                {isDeleting ? "Deleting..." : "Delete"}
              </Button>
            ) : (
              <div />
            )}
            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={isSubmitting || isDeleting}>
                {isSubmitting ? "Saving..." : isEdit ? "Save" : "Add"}
              </Button>
            </div>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
