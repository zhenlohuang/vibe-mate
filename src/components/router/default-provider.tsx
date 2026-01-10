import type { RoutingRule, Provider } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

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
  const handleProviderChange = (providerId: string) => {
    onUpdate({ ...rule, providerId });
  };

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-border bg-card/70 p-3 md:flex-row md:items-center md:gap-3">
      <div className="flex flex-col">
        <span className="text-sm font-semibold text-foreground">
          Default Model Provider
        </span>
      </div>
      <div className="w-full md:w-[220px]">
        <Select value={rule.providerId} onValueChange={handleProviderChange}>
          <SelectTrigger className="bg-secondary/70 border-0">
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
    </div>
  );
}
