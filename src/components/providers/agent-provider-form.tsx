import { useState, useEffect } from "react";
import type { CreateProviderInput, AgentProviderType, Provider } from "@/types";
import { AGENT_TYPES } from "@/lib/constants";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
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
import { ProviderLogo } from "./provider-logo";

interface AgentProviderFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  provider?: Provider;
  onSubmit: (data: CreateProviderInput) => Promise<void>;
  onDelete?: (id: string) => Promise<void>;
  existingAgentTypes?: AgentProviderType[];
}

export function AgentProviderForm({
  open,
  onOpenChange,
  provider,
  onSubmit,
  onDelete,
  existingAgentTypes = [],
}: AgentProviderFormProps) {
  const isEdit = !!provider;
  const [selectedType, setSelectedType] = useState<AgentProviderType | "">(
    (provider?.type as AgentProviderType) || "",
  );
  const [name, setName] = useState(provider?.name || "");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  useEffect(() => {
    if (provider) {
      setSelectedType(provider.type as AgentProviderType);
      setName(provider.name);
    } else {
      setSelectedType("");
      setName("");
    }
  }, [provider]);

  // Filter out agent types that are already added
  const availableAgentTypes = AGENT_TYPES.filter((agent) => {
    if (isEdit) {
      return agent.value === selectedType;
    }
    return !existingAgentTypes.includes(agent.value as AgentProviderType);
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedType || !name.trim()) return;

    setIsSubmitting(true);
    try {
      await onSubmit({
        name: name.trim(),
        category: "Agent",
        type: selectedType,
      });
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to submit agent:", error);
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
      console.error("Failed to delete agent:", error);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      setSelectedType("");
      setName("");
    }
    onOpenChange(isOpen);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[380px]">
        <DialogHeader>
          <DialogTitle className="text-sm">
            {isEdit ? "Edit Agent" : "Add Agent"}
          </DialogTitle>
          <DialogDescription className="text-xs">
            {isEdit
              ? "Update the agent provider details."
              : "Select an AI coding agent to add. You can configure it after adding."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Agent Type Selection */}
          <div className="space-y-2">
            <Label htmlFor="agentType">Agent</Label>
            {availableAgentTypes.length > 0 ? (
              <Select
                value={selectedType}
                onValueChange={(value) => {
                  const nextType = value as AgentProviderType;
                  setSelectedType(nextType);
                  if (!isEdit) {
                    const agentInfo = AGENT_TYPES.find((a) => a.value === nextType);
                    setName((prev) => (prev.trim() ? prev : agentInfo?.label || nextType));
                  }
                }}
                disabled={isEdit}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select an agent" />
                </SelectTrigger>
                <SelectContent>
                  {availableAgentTypes.map((agent) => (
                    <SelectItem key={agent.value} value={agent.value}>
                      <div className="flex items-center gap-2">
                        <ProviderLogo
                          type={agent.value}
                          className="h-4 w-4"
                        />
                        <span>{agent.label}</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <p className="text-sm text-muted-foreground py-2">
                All available agents have been added.
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="agentName">Name</Label>
            <Input
              id="agentName"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter a name for this agent"
              required
            />
          </div>

          <DialogFooter>
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
                onClick={() => handleOpenChange(false)}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={
                  !selectedType ||
                  !name.trim() ||
                  isSubmitting ||
                  isDeleting ||
                  availableAgentTypes.length === 0
                }
              >
                {isSubmitting ? "Saving..." : isEdit ? "Save" : "Add Agent"}
              </Button>
            </div>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
