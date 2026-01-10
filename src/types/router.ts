export interface RoutingRule {
  id: string;
  ruleType: RuleType;
  apiGroup: ApiGroup;
  providerId: string;
  matchPattern: string;
  modelRewrite: string | null;
  priority: number;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export type RuleType = "path" | "model";

export type ApiGroup = "openai" | "anthropic" | "generic";

export interface CreateRuleInput {
  ruleType: RuleType;
  apiGroup: ApiGroup;
  providerId: string;
  matchPattern: string;
  modelRewrite?: string | null;
  enabled?: boolean;
}

export interface UpdateRuleInput {
  ruleType?: RuleType;
  apiGroup?: ApiGroup;
  providerId?: string;
  matchPattern?: string;
  modelRewrite?: string | null;
  enabled?: boolean;
}

export interface ResolvedProvider {
  providerId: string;
  providerName: string;
  apiUrl: string;
  modelName: string;
}
