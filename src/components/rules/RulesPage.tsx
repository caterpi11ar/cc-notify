import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Plus, Trash2, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import {
  useRulesQuery,
  useEventTypesQuery,
  useCreateRule,
  useUpdateRule,
  useDeleteRule,
} from "@/lib/query";

const RULE_TYPES = ["keyword", "regex", "file_change", "custom_event"] as const;

export function RulesPage() {
  const { t } = useTranslation();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [newRuleName, setNewRuleName] = useState("");
  const [newRuleType, setNewRuleType] = useState<string>("");
  const [newRulePattern, setNewRulePattern] = useState("");
  const [newRuleEventTypeId, setNewRuleEventTypeId] = useState("");
  const [newRuleEnabled, setNewRuleEnabled] = useState(true);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const { data: rules, isLoading: rulesLoading } = useRulesQuery();
  const { data: eventTypes, isLoading: eventsLoading } =
    useEventTypesQuery();

  const createRule = useCreateRule();
  const updateRule = useUpdateRule();
  const deleteRule = useDeleteRule();

  const isLoading = rulesLoading || eventsLoading;

  const resetDialog = () => {
    setNewRuleName("");
    setNewRuleType("");
    setNewRulePattern("");
    setNewRuleEventTypeId("");
    setNewRuleEnabled(true);
    setDialogOpen(false);
  };

  const handleCreate = () => {
    if (!newRuleName || !newRuleType || !newRulePattern || !newRuleEventTypeId)
      return;
    createRule.mutate(
      {
        name: newRuleName,
        rule_type: newRuleType,
        pattern: newRulePattern,
        event_type_id: newRuleEventTypeId,
        enabled: newRuleEnabled,
      },
      {
        onSuccess: () => {
          toast.success(t("rules.createSuccess"));
          resetDialog();
        },
      },
    );
  };

  const handleToggle = (id: string, enabled: boolean) => {
    updateRule.mutate(
      { id, rule: { enabled } },
      {
        onSuccess: () => {
          toast.success(t("rules.updateSuccess"));
        },
      },
    );
  };

  const handleDelete = (id: string) => {
    setDeleteTarget(id);
  };

  const confirmDelete = useCallback(() => {
    if (!deleteTarget) return;
    deleteRule.mutate(deleteTarget, {
      onSuccess: () => {
        toast.success(t("rules.deleteSuccess"));
      },
    });
    setDeleteTarget(null);
  }, [deleteTarget, deleteRule, t]);

  const getEventTypeName = (eventTypeId: string): string => {
    const event = eventTypes?.find((e) => e.id === eventTypeId);
    return event?.name ?? eventTypeId;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("rules.title")}</h2>
        <Button size="sm" onClick={() => setDialogOpen(true)}>
          <Plus className="h-4 w-4" />
          {t("rules.add")}
        </Button>
      </div>

      {!rules?.length ? (
        <p className="text-muted-foreground py-8 text-center">
          {t("rules.empty")}
        </p>
      ) : (
        <ScrollArea className="h-[calc(100vh-200px)]">
          <div className="space-y-3 pr-3">
            {rules.map((rule) => (
              <Card key={rule.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2 pt-4 px-4">
                  <div className="flex items-center gap-3">
                    <CardTitle className="text-sm font-medium">
                      {rule.name}
                    </CardTitle>
                    <Badge variant="secondary" className="text-xs">
                      {t(`rules.types.${rule.rule_type}` as const, rule.rule_type)}
                    </Badge>
                  </div>
                  <Switch
                    checked={rule.enabled}
                    onCheckedChange={(checked) =>
                      handleToggle(rule.id, checked)
                    }
                  />
                </CardHeader>
                <CardContent className="px-4 pb-3 pt-0">
                  <div className="flex items-center justify-between">
                    <div className="space-y-1">
                      <p className="text-xs text-muted-foreground">
                        {t("rules.pattern")}:{" "}
                        <code className="bg-muted px-1 py-0.5 rounded text-xs">
                          {rule.pattern}
                        </code>
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {t("rules.eventType")}:{" "}
                        {getEventTypeName(rule.event_type_id)}
                      </p>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      className="text-destructive hover:text-destructive"
                      onClick={() => handleDelete(rule.id)}
                    >
                      <Trash2 className="h-3 w-3" />
                      {t("common.delete")}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </ScrollArea>
      )}

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("rules.add")}</DialogTitle>
            <DialogDescription />
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t("rules.name")}</Label>
              <Input
                value={newRuleName}
                onChange={(e) => setNewRuleName(e.target.value)}
                placeholder={t("rules.name")}
              />
            </div>
            <div className="space-y-2">
              <Label>{t("rules.type")}</Label>
              <Select value={newRuleType} onValueChange={setNewRuleType}>
                <SelectTrigger>
                  <SelectValue placeholder={t("rules.selectType")} />
                </SelectTrigger>
                <SelectContent>
                  {RULE_TYPES.map((type) => (
                    <SelectItem key={type} value={type}>
                      {t(`rules.types.${type}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>{t("rules.pattern")}</Label>
              <Input
                value={newRulePattern}
                onChange={(e) => setNewRulePattern(e.target.value)}
                placeholder={
                  newRuleType === "regex"
                    ? "^error.*$"
                    : newRuleType === "file_change"
                      ? "src/**/*.ts"
                      : "keyword"
                }
              />
            </div>
            <div className="space-y-2">
              <Label>{t("rules.eventType")}</Label>
              <Select
                value={newRuleEventTypeId}
                onValueChange={setNewRuleEventTypeId}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t("rules.selectEventType")} />
                </SelectTrigger>
                <SelectContent>
                  {eventTypes?.map((event) => (
                    <SelectItem key={event.id} value={event.id}>
                      {event.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center justify-between">
              <Label>{t("common.enabled")}</Label>
              <Switch
                checked={newRuleEnabled}
                onCheckedChange={setNewRuleEnabled}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={resetDialog}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={handleCreate}
              disabled={
                !newRuleName ||
                !newRuleType ||
                !newRulePattern ||
                !newRuleEventTypeId ||
                createRule.isPending
              }
            >
              {createRule.isPending && (
                <Loader2 className="h-4 w-4 animate-spin" />
              )}
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <ConfirmDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}
        description={t("rules.deleteConfirm")}
        onConfirm={confirmDelete}
        variant="destructive"
      />
    </div>
  );
}
