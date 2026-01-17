import { useState } from "react";
import type { CreateProviderInput, AgentProviderType } from "@/types";
import { AGENT_TYPES } from "@/lib/constants";
import { Button } from "@/components/ui/button";
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
import { ProviderLogo } from "./provider-logo";

interface AgentProviderFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: CreateProviderInput) => Promise<void>;
  existingAgentTypes?: AgentProviderType[];
}

export function AgentProviderForm({
  open,
  onOpenChange,
  onSubmit,
  existingAgentTypes = [],
}: AgentProviderFormProps) {
  const [selectedType, setSelectedType] = useState<AgentProviderType | "">("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Filter out agent types that are already added
  const availableAgentTypes = AGENT_TYPES.filter(
    (agent) => !existingAgentTypes.includes(agent.value as AgentProviderType)
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedType) return;

    setIsSubmitting(true);
    try {
      const agentInfo = AGENT_TYPES.find((a) => a.value === selectedType);
      await onSubmit({
        name: agentInfo?.label || selectedType,
        category: "Agent",
        type: selectedType,
      });
      onOpenChange(false);
      setSelectedType("");
    } catch (error) {
      console.error("Failed to add agent:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      setSelectedType("");
    }
    onOpenChange(isOpen);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[380px]">
        <DialogHeader>
          <DialogTitle className="text-sm">Add Agent</DialogTitle>
          <DialogDescription className="text-xs">
            Select an AI coding agent to add. You can configure it after adding.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Agent Type Selection */}
          <div className="space-y-2">
            <Label htmlFor="agentType">Agent</Label>
            {availableAgentTypes.length > 0 ? (
              <Select
                value={selectedType}
                onValueChange={(value) =>
                  setSelectedType(value as AgentProviderType)
                }
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

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={!selectedType || isSubmitting || availableAgentTypes.length === 0}
            >
              {isSubmitting ? "Adding..." : "Add Agent"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
