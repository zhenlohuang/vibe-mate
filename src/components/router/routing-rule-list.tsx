import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import type { RoutingRule, Provider } from "@/types";
import { RoutingRuleItem } from "./routing-rule-item";

interface RoutingRuleListProps {
  rules: RoutingRule[];
  providers: Provider[];
  onUpdateRule: (rule: RoutingRule) => void;
  onDeleteRule: (id: string) => void;
  onDuplicateRule: (rule: RoutingRule) => void;
  onReorderRules: (ruleIds: string[]) => void;
}

export function RoutingRuleList({
  rules,
  providers,
  onUpdateRule,
  onDeleteRule,
  onDuplicateRule,
  onReorderRules,
}: RoutingRuleListProps) {
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = rules.findIndex((r) => r.id === active.id);
      const newIndex = rules.findIndex((r) => r.id === over.id);

      // Create new order
      const newRules = [...rules];
      const [removed] = newRules.splice(oldIndex, 1);
      newRules.splice(newIndex, 0, removed);

      onReorderRules(newRules.map((r) => r.id));
    }
  };

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
      modifiers={[restrictToVerticalAxis]}
    >
      <SortableContext
        items={rules.map((r) => r.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-3">
          {rules.map((rule) => (
            <RoutingRuleItem
              key={rule.id}
              rule={rule}
              providers={providers}
              onUpdate={onUpdateRule}
              onDelete={onDeleteRule}
              onDuplicate={onDuplicateRule}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}
