import { useEffect, useMemo, useState } from "react";
import { Loader2, Plus } from "lucide-react";
import { MainContent } from "@/components/layout/main-content";
import {
  RoutingRuleList,
  NewRuleItem,
  DefaultProvider,
} from "@/components/router";
import { Button } from "@/components/ui/button";
import { useRoutingRules } from "@/hooks/use-routing-rules";
import { useProviders } from "@/hooks/use-providers";
import { useToast } from "@/hooks/use-toast";
import type {
  RoutingRule,
  CreateRuleInput,
  UpdateRuleInput,
  RuleType,
  ApiGroup,
} from "@/types";

export function RouterPage() {
  const { rules, isLoading, createRule, updateRule, deleteRule, reorderRules } =
    useRoutingRules();
  const { providers } = useProviders();
  const { toast } = useToast();
  const [addingRule, setAddingRule] = useState<{
    apiGroup: ApiGroup;
    ruleType: RuleType;
  } | null>(null);

  const defaultProviderId = providers[0]?.id || "";

  const openaiPathPattern = "/api/openai/*";
  const anthropicPathPattern = "/api/anthropic/*";
  const genericPathPattern = "/api/*";

  const openaiPathRule = useMemo(
    () =>
      rules.find(
        (rule) =>
          rule.apiGroup === "openai" &&
          rule.ruleType === "path" &&
          rule.matchPattern === openaiPathPattern
      ),
    [rules, openaiPathPattern]
  );

  const anthropicPathRule = useMemo(
    () =>
      rules.find(
        (rule) =>
          rule.apiGroup === "anthropic" &&
          rule.ruleType === "path" &&
          rule.matchPattern === anthropicPathPattern
      ),
    [rules, anthropicPathPattern]
  );

  const genericPathRules = useMemo(
    () =>
      rules
        .filter((rule) => rule.apiGroup === "generic" && rule.ruleType === "path")
        .sort((a, b) => {
          const aDefault = a.matchPattern === genericPathPattern;
          const bDefault = b.matchPattern === genericPathPattern;
          if (aDefault !== bDefault) {
            return aDefault ? 1 : -1;
          }
          return a.priority - b.priority;
        }),
    [rules]
  );

  const openaiModelRules = useMemo(
    () =>
      rules
        .filter((rule) => rule.apiGroup === "openai" && rule.ruleType === "model")
        .sort((a, b) => a.priority - b.priority),
    [rules]
  );

  const anthropicModelRules = useMemo(
    () =>
      rules
        .filter((rule) => rule.apiGroup === "anthropic" && rule.ruleType === "model")
        .sort((a, b) => a.priority - b.priority),
    [rules]
  );

  const openaiHasRules = useMemo(
    () => rules.some((rule) => rule.apiGroup === "openai"),
    [rules]
  );

  const anthropicHasRules = useMemo(
    () => rules.some((rule) => rule.apiGroup === "anthropic"),
    [rules]
  );

  useEffect(() => {
    if (isLoading || !defaultProviderId) {
      return;
    }

    const pending: CreateRuleInput[] = [];

    if (!openaiHasRules) {
      pending.push({
        ruleType: "path",
        apiGroup: "openai",
        providerId: defaultProviderId,
        matchPattern: openaiPathPattern,
        enabled: true,
      });
    }

    if (!anthropicHasRules) {
      pending.push({
        ruleType: "path",
        apiGroup: "anthropic",
        providerId: defaultProviderId,
        matchPattern: anthropicPathPattern,
        enabled: true,
      });
    }

    if (!pending.length) {
      return;
    }

    const createDefaults = async () => {
      try {
        for (const input of pending) {
          await createRule(input);
        }
      } catch (error) {
        toast({
          title: "Error",
          description: String(error),
          variant: "destructive",
        });
      }
    };

    createDefaults();
  }, [
    isLoading,
    defaultProviderId,
    openaiHasRules,
    anthropicHasRules,
    createRule,
    toast,
  ]);

  const handleStartAddRule = (apiGroup: ApiGroup, ruleType: RuleType) => {
    const targetProvider = providers[0];
    if (!targetProvider) {
      toast({
        title: "No Provider Available",
        description: "Please add a provider before creating routing rules.",
        variant: "destructive",
      });
      return;
    }
    setAddingRule({ apiGroup, ruleType });
  };

  const handleConfirmAddRule = async (input: CreateRuleInput) => {
    if (
      input.apiGroup === "generic" &&
      input.ruleType === "path" &&
      (input.matchPattern.startsWith("/api/openai") ||
        input.matchPattern.startsWith("/api/anthropic"))
    ) {
      toast({
        title: "Invalid Path Pattern",
        description: "Generic path rules cannot start with /api/openai or /api/anthropic.",
        variant: "destructive",
      });
      return;
    }

    try {
      await createRule(input);
      setAddingRule(null);
      toast({
        title: "Rule Created",
        description: "A new routing rule has been added.",
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleCancelAddRule = () => {
    setAddingRule(null);
  };

  const handleDuplicateRule = async (rule: RoutingRule) => {
    const input: CreateRuleInput = {
      ruleType: rule.ruleType,
      apiGroup: rule.apiGroup,
      providerId: rule.providerId,
      matchPattern: rule.matchPattern,
      modelRewrite: rule.modelRewrite || undefined,
      enabled: true,
    };

    try {
      await createRule(input);
      toast({
        title: "Rule Duplicated",
        description: "The routing rule has been duplicated.",
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleUpdateRule = async (rule: RoutingRule) => {
    const input: UpdateRuleInput = {
      ruleType: rule.ruleType,
      apiGroup: rule.apiGroup,
      providerId: rule.providerId,
      matchPattern: rule.matchPattern,
      modelRewrite: rule.modelRewrite,
      enabled: rule.enabled,
    };

    try {
      await updateRule(rule.id, input);
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleDeleteRule = async (id: string) => {
    try {
      await deleteRule(id);
      toast({
        title: "Rule Deleted",
        description: "The routing rule has been removed.",
      });
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const handleReorderRules = async (ruleIds: string[]) => {
    try {
      await reorderRules(ruleIds);
    } catch (error) {
      toast({
        title: "Error",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  if (isLoading) {
    return (
      <MainContent
        title="Routing Rules"
        description="Route API requests by model or path to your providers."
      >
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
        </div>
      </MainContent>
    );
  }

  return (
    <MainContent
      title="Routing Rules"
      description="Route API requests by model or path to your providers."
    >
      <div className="space-y-4 pb-12">
        <div className="rounded-xl border border-border bg-card/50 p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold">OpenAI Compatible API</h3>
            </div>
            <Button
              onClick={() => handleStartAddRule("openai", "model")}
              className="h-8 gap-2 border border-primary px-3"
            >
              <Plus className="h-4 w-4" />
              Routing Rule
            </Button>
          </div>
          <div className="mt-4 space-y-3">
            {openaiPathRule && (
              <DefaultProvider
                rule={openaiPathRule}
                providers={providers}
                onUpdate={handleUpdateRule}
              />
            )}
            {openaiModelRules.length > 0 && (
              <RoutingRuleList
                rules={openaiModelRules}
                providers={providers}
                onUpdateRule={handleUpdateRule}
                onDeleteRule={handleDeleteRule}
                onDuplicateRule={handleDuplicateRule}
                onReorderRules={handleReorderRules}
              />
            )}
            {addingRule?.apiGroup === "openai" && addingRule.ruleType === "model" && (
              <NewRuleItem
                providers={providers}
                defaultProviderId={defaultProviderId}
                ruleType="model"
                apiGroup="openai"
                onConfirm={handleConfirmAddRule}
                onCancel={handleCancelAddRule}
              />
            )}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-card/50 p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold">Anthropic Compatible API</h3>
            </div>
            <Button
              onClick={() => handleStartAddRule("anthropic", "model")}
              className="h-8 gap-2 border border-primary px-3"
            >
              <Plus className="h-4 w-4" />
              Routing Rule
            </Button>
          </div>
          <div className="mt-4 space-y-3">
            {anthropicPathRule && (
              <DefaultProvider
                rule={anthropicPathRule}
                providers={providers}
                onUpdate={handleUpdateRule}
              />
            )}
            {anthropicModelRules.length > 0 && (
              <RoutingRuleList
                rules={anthropicModelRules}
                providers={providers}
                onUpdateRule={handleUpdateRule}
                onDeleteRule={handleDeleteRule}
                onDuplicateRule={handleDuplicateRule}
                onReorderRules={handleReorderRules}
              />
            )}
            {addingRule?.apiGroup === "anthropic" && addingRule.ruleType === "model" && (
              <NewRuleItem
                providers={providers}
                defaultProviderId={defaultProviderId}
                ruleType="model"
                apiGroup="anthropic"
                onConfirm={handleConfirmAddRule}
                onCancel={handleCancelAddRule}
              />
            )}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-card/50 p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold">Generic API</h3>
            </div>
            <Button
              onClick={() => handleStartAddRule("generic", "path")}
              className="h-8 gap-2 border border-primary px-3"
            >
              <Plus className="h-4 w-4" />
              Routing Rule
            </Button>
          </div>
          <div className="mt-4 space-y-3">
            {genericPathRules.length > 0 && (
              <RoutingRuleList
                rules={genericPathRules}
                providers={providers}
                onUpdateRule={handleUpdateRule}
                onDeleteRule={handleDeleteRule}
                onDuplicateRule={handleDuplicateRule}
                onReorderRules={handleReorderRules}
              />
            )}
            {addingRule?.apiGroup === "generic" && addingRule.ruleType === "path" && (
              <NewRuleItem
                providers={providers}
                defaultProviderId={defaultProviderId}
                ruleType="path"
                apiGroup="generic"
                initialMatchPattern="/api/custom/*"
                onConfirm={handleConfirmAddRule}
                onCancel={handleCancelAddRule}
              />
            )}
          </div>
        </div>
      </div>
    </MainContent>
  );
}
