import { useState, useEffect } from "react";
import { Check, X } from "lucide-react";
import type { RoutingRule, Provider } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface DefaultProviderProps {
  rule: RoutingRule;
  providers: Provider[];
  onUpdate: (rule: RoutingRule) => void;
}

export function DefaultProvider({
  rule,
  providers,
  onUpdate,
}: DefaultProviderProps) {
  const [providerId, setProviderId] = useState(rule.providerId);

  useEffect(() => {
    setProviderId(rule.providerId);
  }, [rule.providerId]);

  const hasChanges = providerId !== rule.providerId;

  const handleSave = () => {
    onUpdate({ ...rule, providerId });
  };

  const handleCancel = () => {
    setProviderId(rule.providerId);
  };

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-border bg-card/70 p-3 md:flex-row md:items-center md:gap-3">
      <div className="flex flex-col">
        <span className="text-xs font-semibold tracking-wide text-muted-foreground">
          Default Model Provider
        </span>
      </div>
      <div className="w-full md:w-[220px]">
        <Select value={providerId} onValueChange={setProviderId}>
          <SelectTrigger className="h-7 text-xs bg-secondary/70 border-0">
            <SelectValue placeholder="Select provider" />
          </SelectTrigger>
          <SelectContent>
            {providers.map((provider) => (
              <SelectItem key={provider.id} value={provider.id}>
                {provider.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {hasChanges && (
        <TooltipProvider delayDuration={300}>
          <div className="flex items-center gap-1">
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={handleSave}
                  className="p-1.5 rounded-md text-primary hover:text-primary hover:bg-secondary transition-colors"
                >
                  <Check className="h-3.5 w-3.5" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p className="text-sm">Save</p>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={handleCancel}
                  className="p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                >
                  <X className="h-3.5 w-3.5" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p className="text-sm">Cancel</p>
              </TooltipContent>
            </Tooltip>
          </div>
        </TooltipProvider>
      )}
    </div>
  );
}
