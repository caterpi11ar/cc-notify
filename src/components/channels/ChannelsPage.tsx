import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  Plus,
  Trash2,
  Zap,
  Loader2,
  Pencil,
} from "lucide-react";
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
import {
  useChannelsQuery,
  useCreateChannel,
  useUpdateChannel,
  useDeleteChannel,
  useTestChannel,
} from "@/lib/query";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import type { Channel } from "@/types";

const CHANNEL_TYPES = [
  "native",
  "slack",
  "discord",
  "teams",
  "telegram",
  "webhook",
  "sound",
  "voice",
  "tray_badge",
] as const;

type ChannelType = (typeof CHANNEL_TYPES)[number];

const WEBHOOK_TEMPLATES = ["generic", "feishu"] as const;
type WebhookTemplate = (typeof WEBHOOK_TEMPLATES)[number];

interface ChannelConfigField {
  key: string;
  label: string;
  placeholder: string;
  type?: "text" | "number" | "textarea" | "select";
  options?: { value: string; label: string }[];
}

function getConfigFields(channelType: ChannelType, template?: string): ChannelConfigField[] {
  switch (channelType) {
    case "slack":
      return [
        {
          key: "webhook_url",
          label: "Webhook URL",
          placeholder: "https://hooks.slack.com/services/...",
        },
        {
          key: "channel",
          label: "Channel",
          placeholder: "#general",
        },
        {
          key: "mention",
          label: "Mention",
          placeholder: "@here, @channel, or @username",
        },
      ];
    case "discord":
      return [
        {
          key: "webhook_url",
          label: "Webhook URL",
          placeholder: "https://discord.com/api/webhooks/...",
        },
        {
          key: "username",
          label: "Username",
          placeholder: "Bot display name",
        },
        {
          key: "avatar_url",
          label: "Avatar URL",
          placeholder: "https://example.com/avatar.png",
        },
        {
          key: "embed_color",
          label: "Embed Color",
          placeholder: "#5865F2",
        },
      ];
    case "telegram":
      return [
        {
          key: "bot_token",
          label: "Bot Token",
          placeholder: "123456:ABC-DEF...",
        },
        {
          key: "chat_id",
          label: "Chat ID",
          placeholder: "-1001234567890",
        },
        {
          key: "parse_mode",
          label: "Parse Mode",
          placeholder: "",
          type: "select",
          options: [
            { value: "HTML", label: "HTML" },
            { value: "Markdown", label: "Markdown" },
            { value: "MarkdownV2", label: "MarkdownV2" },
          ],
        },
      ];
    case "teams":
      return [
        {
          key: "webhook_url",
          label: "Webhook URL",
          placeholder: "https://outlook.office.com/webhook/...",
        },
      ];
    case "webhook":
      if (template === "feishu") {
        return [
          {
            key: "webhook_url",
            label: "Webhook URL",
            placeholder: "https://open.feishu.cn/open-apis/bot/v2/hook/...",
          },
        ];
      }
      return [
        {
          key: "url",
          label: "URL",
          placeholder: "https://example.com/webhook",
        },
        {
          key: "method",
          label: "HTTP Method",
          placeholder: "",
          type: "select",
          options: [
            { value: "POST", label: "POST" },
            { value: "GET", label: "GET" },
            { value: "PUT", label: "PUT" },
            { value: "PATCH", label: "PATCH" },
          ],
        },
        {
          key: "headers",
          label: "Headers (JSON)",
          placeholder: '{"Content-Type": "application/json"}',
          type: "textarea",
        },
        {
          key: "body_template",
          label: "Body Template",
          placeholder: '{"text": "{{message}}"}',
          type: "textarea",
        },
      ];
    case "sound":
      return [
        {
          key: "sound_file",
          label: "Sound File",
          placeholder: "default, Ping, Glass, Basso...",
        },
        {
          key: "volume",
          label: "Volume",
          placeholder: "0.0 - 1.0",
          type: "number",
        },
      ];
    case "voice":
      return [
        {
          key: "voice",
          label: "Voice Name",
          placeholder: "Samantha, Alex, Victoria...",
        },
      ];
    case "native":
      return [
        {
          key: "timeout",
          label: "Timeout (seconds)",
          placeholder: "10",
          type: "number",
        },
      ];
    case "tray_badge":
      return [];
    default:
      return [];
  }
}

interface ChannelFormState {
  name: string;
  channel_type: ChannelType | "";
  config: Record<string, string>;
}

const emptyForm: ChannelFormState = {
  name: "",
  channel_type: "",
  config: {},
};

export function ChannelsPage() {
  const { t } = useTranslation();

  // Dialog state
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingChannel, setEditingChannel] = useState<Channel | null>(null);
  const [form, setForm] = useState<ChannelFormState>(emptyForm);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  // Queries and mutations
  const { data: channels, isLoading } = useChannelsQuery();
  const createChannel = useCreateChannel();
  const updateChannel = useUpdateChannel();
  const deleteChannel = useDeleteChannel();
  const testChannel = useTestChannel();

  const isEditing = editingChannel !== null;

  const resetDialog = () => {
    setForm(emptyForm);
    setEditingChannel(null);
    setDialogOpen(false);
  };

  const openCreateDialog = () => {
    setEditingChannel(null);
    setForm(emptyForm);
    setDialogOpen(true);
  };

  const openEditDialog = (channel: Channel) => {
    setEditingChannel(channel);
    const configAsStrings: Record<string, string> = {};
    for (const [key, value] of Object.entries(channel.config)) {
      configAsStrings[key] = typeof value === "string" ? value : JSON.stringify(value);
    }
    setForm({
      name: channel.name,
      channel_type: channel.channel_type as ChannelType,
      config: configAsStrings,
    });
    setDialogOpen(true);
  };

  const handleSave = () => {
    if (!form.name || !form.channel_type) return;

    // Build config, converting number fields appropriately
    const config: Record<string, unknown> = { ...form.config };
    const webhookTemplate = form.channel_type === "webhook" ? (form.config.template || "generic") : undefined;
    const fields = getConfigFields(form.channel_type as ChannelType, webhookTemplate);
    for (const field of fields) {
      if (field.type === "number" && config[field.key] != null) {
        const num = parseFloat(config[field.key] as string);
        if (!isNaN(num)) {
          config[field.key] = num;
        }
      }
      if (field.key === "headers" && typeof config[field.key] === "string") {
        try {
          config[field.key] = JSON.parse(config[field.key] as string);
        } catch {
          // keep as string if invalid JSON
        }
      }
    }

    if (isEditing) {
      updateChannel.mutate(
        {
          id: editingChannel.id,
          channel: {
            name: form.name,
            channel_type: form.channel_type,
            config,
          },
        },
        {
          onSuccess: () => {
            toast.success(t("channels.updateSuccess"));
            resetDialog();
          },
        },
      );
    } else {
      createChannel.mutate(
        {
          name: form.name,
          channel_type: form.channel_type,
          config,
          enabled: true,
          sort_index: (channels?.length ?? 0) + 1,
        },
        {
          onSuccess: () => {
            toast.success(t("channels.createSuccess"));
            resetDialog();
          },
        },
      );
    }
  };

  const handleToggle = (id: string, enabled: boolean) => {
    updateChannel.mutate(
      { id, channel: { enabled } },
      {
        onSuccess: () => {
          toast.success(t("channels.updateSuccess"));
        },
      },
    );
  };

  const handleDelete = (id: string) => {
    setDeleteTarget(id);
  };

  const confirmDelete = useCallback(() => {
    if (!deleteTarget) return;
    deleteChannel.mutate(deleteTarget, {
      onSuccess: () => {
        toast.success(t("channels.deleteSuccess"));
      },
    });
    setDeleteTarget(null);
  }, [deleteTarget, deleteChannel, t]);

  const handleTest = (id: string) => {
    setTestingId(id);
    testChannel.mutate(id, {
      onSuccess: (result) => {
        if (result.success) {
          toast.success(t("channels.testSuccess"));
        } else {
          toast.error(
            t("channels.testFailed", { error: result.message ?? "Unknown" }),
          );
        }
        setTestingId(null);
      },
      onError: (error: Error) => {
        toast.error(t("channels.testFailed", { error: error.message }));
        setTestingId(null);
      },
    });
  };

  const setConfigValue = (key: string, value: string) => {
    setForm((prev) => ({
      ...prev,
      config: { ...prev.config, [key]: value },
    }));
  };

  const isSaving = createChannel.isPending || updateChannel.isPending;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const webhookTemplate = form.channel_type === "webhook" ? (form.config.template || "generic") : undefined;
  const configFields = form.channel_type
    ? getConfigFields(form.channel_type as ChannelType, webhookTemplate)
    : [];

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("channels.title")}</h2>
        <Button size="sm" onClick={openCreateDialog}>
          <Plus className="h-4 w-4" />
          {t("channels.add")}
        </Button>
      </div>

      {/* Channel list */}
      {!channels?.length ? (
        <p className="text-muted-foreground py-8 text-center">
          {t("channels.empty")}
        </p>
      ) : (
        <ScrollArea className="h-[calc(100vh-200px)]">
          <div className="space-y-3 pr-3">
            {channels.map((channel) => (
              <Card key={channel.id}>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2 pt-4 px-4">
                  <div className="flex items-center gap-3">
                    <CardTitle className="text-sm font-medium">
                      {channel.name}
                    </CardTitle>
                    <Badge variant="secondary" className="text-xs">
                      {channel.channel_type === "webhook" && channel.config?.template
                        ? `${t(`channels.types.webhook`)} (${t(`channels.templates.${channel.config.template}` as const, String(channel.config.template))})`
                        : t(
                            `channels.types.${channel.channel_type}` as const,
                            channel.channel_type,
                          )}
                    </Badge>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">
                      {channel.enabled
                        ? t("channels.enabled")
                        : t("channels.disabled")}
                    </span>
                    <Switch
                      checked={channel.enabled}
                      onCheckedChange={(checked) =>
                        handleToggle(channel.id, checked)
                      }
                    />
                  </div>
                </CardHeader>
                <CardContent className="px-4 pb-3 pt-0">
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleTest(channel.id)}
                      disabled={testingId === channel.id || !channel.enabled}
                    >
                      {testingId === channel.id ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : (
                        <Zap className="h-3 w-3" />
                      )}
                      {t("channels.test")}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => openEditDialog(channel)}
                    >
                      <Pencil className="h-3 w-3" />
                      {t("common.edit")}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="text-destructive hover:text-destructive"
                      onClick={() => handleDelete(channel.id)}
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

      {/* Create / Edit Dialog */}
      <Dialog open={dialogOpen} onOpenChange={(open) => { if (!open) resetDialog(); }}>
        <DialogContent className="max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {isEditing ? t("common.edit") : t("channels.add")}
            </DialogTitle>
            <DialogDescription />
          </DialogHeader>

          <div className="space-y-4">
            {/* Channel Name */}
            <div className="space-y-2">
              <Label>{t("channels.name")}</Label>
              <Input
                value={form.name}
                onChange={(e) =>
                  setForm((prev) => ({ ...prev, name: e.target.value }))
                }
                placeholder={t("channels.name")}
              />
            </div>

            {/* Channel Type */}
            <div className="space-y-2">
              <Label>{t("channels.type")}</Label>
              <Select
                value={form.channel_type}
                onValueChange={(v) => {
                  setForm((prev) => ({
                    ...prev,
                    channel_type: v as ChannelType,
                    config: isEditing && prev.channel_type === v ? prev.config : {},
                  }));
                }}
                disabled={isEditing}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t("channels.selectType")} />
                </SelectTrigger>
                <SelectContent>
                  {CHANNEL_TYPES.map((type) => (
                    <SelectItem key={type} value={type}>
                      {t(`channels.types.${type}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Webhook Template Selector */}
            {form.channel_type === "webhook" && (
              <div className="space-y-2">
                <Label>{t("channels.template")}</Label>
                <Select
                  value={form.config.template || "generic"}
                  onValueChange={(v) => {
                    // Clear template-specific config fields when switching
                    const newConfig: Record<string, string> = { template: v };
                    setForm((prev) => ({
                      ...prev,
                      config: newConfig,
                    }));
                  }}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {WEBHOOK_TEMPLATES.map((tmpl) => (
                      <SelectItem key={tmpl} value={tmpl}>
                        {t(`channels.templates.${tmpl}`)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {/* Dynamic Config Fields */}
            {configFields.length > 0 && (
              <div className="space-y-4">
                <Label className="text-sm font-semibold">
                  {t("channels.config")}
                </Label>
                {configFields.map((field) => (
                  <div key={field.key} className="space-y-2">
                    <Label className="text-xs text-muted-foreground">
                      {field.label}
                    </Label>
                    {field.type === "select" && field.options ? (
                      <Select
                        value={form.config[field.key] ?? ""}
                        onValueChange={(v) => setConfigValue(field.key, v)}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder={field.placeholder || `Select ${field.label}`} />
                        </SelectTrigger>
                        <SelectContent>
                          {field.options.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                              {opt.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    ) : field.type === "textarea" ? (
                      <textarea
                        className="flex min-h-[80px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                        value={form.config[field.key] ?? ""}
                        onChange={(e) => setConfigValue(field.key, e.target.value)}
                        placeholder={field.placeholder}
                      />
                    ) : (
                      <Input
                        type={field.type === "number" ? "number" : "text"}
                        step={field.type === "number" ? "any" : undefined}
                        value={form.config[field.key] ?? ""}
                        onChange={(e) => setConfigValue(field.key, e.target.value)}
                        placeholder={field.placeholder}
                      />
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={resetDialog}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={handleSave}
              disabled={!form.name || !form.channel_type || isSaving}
            >
              {isSaving && <Loader2 className="h-4 w-4 animate-spin" />}
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <ConfirmDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}
        description={t("channels.deleteConfirm")}
        onConfirm={confirmDelete}
        variant="destructive"
      />
    </div>
  );
}
