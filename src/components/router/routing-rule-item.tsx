import { useState } from "react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, ArrowRight, Trash2, Copy, Check, X } from "lucide-react";
import type {
  RoutingRule,
  Provider,
  CreateRuleInput,
  RuleType,
  ApiGroup,
} from "@/types";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
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
import { Switch } from "@/components/ui/switch";

interface RoutingRuleItemProps {
  rule: RoutingRule;
  providers: Provider[];
  onUpdate: (rule: RoutingRule) => void;
  onDelete: (id: string) => void;
  onDuplicate: (rule: RoutingRule) => void;
}

const lockedPathPatterns = new Set(["/api/openai/*", "/api/anthropic/*", "/api/*"]);

function isLockedRule(rule: RoutingRule) {
  return rule.ruleType === "path" && lockedPathPatterns.has(rule.matchPattern);
}

function getRuleLabel(ruleType: RuleType) {
  return ruleType === "path" ? "PATH" : "MODEL";
}

function getRulePlaceholder(ruleType: RuleType) {
  return ruleType === "path" ? "/api/*" : "gpt-4*";
}

export function RoutingRuleItem({
  rule,
  providers,
  onUpdate,
  onDelete,
  onDuplicate,
}: RoutingRuleItemProps) {
  const isLocked = isLockedRule(rule);
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: rule.id, disabled: isLocked });

  const style: React.CSSProperties = {
    transform: CSS.Translate.toString(transform),
    transition,
  };

  const label = getRuleLabel(rule.ruleType);
  const placeholder = getRulePlaceholder(rule.ruleType);

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex items-center gap-2 rounded-lg border border-border bg-card p-3 transition-all",
        isDragging && "routing-rule-dragging z-50 shadow-2xl",
        !rule.enabled && "opacity-50"
      )}
    >
      {/* Drag Handle */}
      <button
        className="cursor-grab touch-none text-muted-foreground/50 hover:text-muted-foreground transition-colors"
        {...attributes}
        {...listeners}
      >
        <GripVertical className="h-4 w-4" />
      </button>

      <div className="text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/70 w-10 text-center">
        {label}
      </div>

      {/* Match Pattern */}
      <div className="flex-1 max-w-[160px]">
        <Input
          className="font-mono text-[11px] bg-secondary/70 border-0"
          placeholder={placeholder}
          value={rule.matchPattern}
          onChange={(e) => onUpdate({ ...rule, matchPattern: e.target.value })}
          disabled={isLocked}
        />
      </div>

      {/* Arrow */}
      <ArrowRight className="h-3.5 w-3.5 flex-shrink-0 text-primary" />

      {/* Target Provider */}
      <div className="w-[130px]">
        <Select
          value={rule.providerId}
          onValueChange={(providerId) => onUpdate({ ...rule, providerId })}
        >
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

      {/* Model Rewrite */}
      <div className="flex-1">
        <Input
          className="font-mono text-[11px] bg-secondary/70 border-0"
          placeholder="(Optional)"
          value={rule.modelRewrite || ""}
          onChange={(e) =>
            onUpdate({ ...rule, modelRewrite: e.target.value || null })
          }
        />
      </div>

      {/* Action Buttons */}
      <TooltipProvider delayDuration={300}>
        <div className="flex items-center gap-1">
          {!isLocked && (
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={() => onDuplicate(rule)}
                  className="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
                >
                  <Copy className="h-3.5 w-3.5" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p className="text-xs">Duplicate</p>
              </TooltipContent>
            </Tooltip>
          )}

          {!isLocked && (
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  onClick={() => onDelete(rule.id)}
                  className="p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p className="text-xs">Delete</p>
              </TooltipContent>
            </Tooltip>
          )}

          <div className="flex items-center gap-1.5 pl-1">
            <Switch
              checked={rule.enabled}
              onCheckedChange={(enabled) => onUpdate({ ...rule, enabled })}
              disabled={isLocked}
            />
          </div>
        </div>
      </TooltipProvider>
    </div>
  );
}

interface NewRuleItemProps {
  providers: Provider[];
  defaultProviderId: string;
  ruleType: RuleType;
  apiGroup: ApiGroup;
  matchPatternLocked?: boolean;
  initialMatchPattern?: string;
  onConfirm: (input: CreateRuleInput) => void;
  onCancel: () => void;
}

export function NewRuleItem({
  providers,
  defaultProviderId,
  ruleType,
  apiGroup,
  matchPatternLocked = false,
  initialMatchPattern,
  onConfirm,
  onCancel,
}: NewRuleItemProps) {
  const [matchPattern, setMatchPattern] = useState(
    initialMatchPattern || (ruleType === "path" ? "/api/*" : "*")
  );
  const [providerId, setProviderId] = useState(defaultProviderId);
  const [modelRewrite, setModelRewrite] = useState("");

  const label = getRuleLabel(ruleType);
  const placeholder = getRulePlaceholder(ruleType);

  const handleConfirm = () => {
    onConfirm({
      ruleType,
      apiGroup,
      providerId,
      matchPattern,
      modelRewrite: modelRewrite || undefined,
      enabled: true,
    });
  };

  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-lg border border-border bg-card p-3 transition-all",
        "animate-in fade-in slide-in-from-top-2 duration-200"
      )}
    >
      <div className="cursor-not-allowed text-muted-foreground/40">
        <GripVertical className="h-4 w-4" />
      </div>

      <div className="text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/70 w-10 text-center">
        {label}
      </div>

      <div className="flex-1 max-w-[160px]">
        <Input
          className="font-mono text-[11px] bg-secondary/70 border-0"
          placeholder={placeholder}
          value={matchPattern}
          onChange={(e) => setMatchPattern(e.target.value)}
          autoFocus={!matchPatternLocked}
          disabled={matchPatternLocked}
        />
      </div>

      <ArrowRight className="h-3.5 w-3.5 flex-shrink-0 text-primary" />

      <div className="w-[130px]">
        <Select value={providerId} onValueChange={setProviderId}>
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

      <div className="flex-1">
        <Input
          className="font-mono text-[11px] bg-secondary/70 border-0"
          placeholder="(Optional)"
          value={modelRewrite}
          onChange={(e) => setModelRewrite(e.target.value)}
        />
      </div>

      <TooltipProvider delayDuration={300}>
        <div className="flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={handleConfirm}
                className="p-1.5 rounded-md text-primary hover:text-primary hover:bg-secondary transition-colors"
              >
                <Check className="h-3.5 w-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p className="text-xs">Confirm</p>
            </TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onCancel}
                className="p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p className="text-xs">Cancel</p>
            </TooltipContent>
          </Tooltip>
        </div>
      </TooltipProvider>
    </div>
  );
}
