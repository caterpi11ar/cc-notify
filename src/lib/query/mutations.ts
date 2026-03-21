import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  channelsApi,
  eventTypesApi,
  rulesApi,
  routingApi,
  templatesApi,
  historyApi,
  settingsApi,
  hooksApi,
} from "@/lib/api";
import type { Channel, EventType, Rule, Routing, Template } from "@/types";

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "An unknown error occurred";
}

// ── Channels ──

export const useCreateChannel = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (channel: Omit<Channel, "id" | "created_at" | "updated_at">) =>
      channelsApi.create(channel),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useUpdateChannel = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, channel }: { id: string; channel: Partial<Channel> }) =>
      channelsApi.update(id, channel),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteChannel = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => channelsApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useTestChannel = () => {
  return useMutation({
    mutationFn: (id: string) => channelsApi.test(id),
  });
};

// ── Event Types ──

export const useCreateEventType = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (eventType: Omit<EventType, "id">) =>
      eventTypesApi.create(eventType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["eventTypes"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useUpdateEventType = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      eventType,
    }: {
      id: string;
      eventType: Partial<EventType>;
    }) => eventTypesApi.update(id, eventType),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["eventTypes"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteEventType = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => eventTypesApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["eventTypes"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── Rules ──

export const useCreateRule = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (rule: Omit<Rule, "id" | "created_at">) =>
      rulesApi.create(rule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rules"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useUpdateRule = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, rule }: { id: string; rule: Partial<Rule> }) =>
      rulesApi.update(id, rule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rules"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteRule = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => rulesApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["rules"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── Routing ──

export const useSetRouting = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (routing: Routing) => routingApi.set(routing),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["routing"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteRouting = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      eventTypeId,
      channelId,
    }: {
      eventTypeId: string;
      channelId: string;
    }) => routingApi.delete(eventTypeId, channelId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["routing"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── Templates ──

export const useCreateTemplate = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (template: Omit<Template, "id">) =>
      templatesApi.create(template),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["templates"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useUpdateTemplate = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      template,
    }: {
      id: string;
      template: Partial<Template>;
    }) => templatesApi.update(id, template),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["templates"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteTemplate = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => templatesApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["templates"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── History ──

export const useClearHistory = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => historyApi.clear(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── Settings ──

export const useSetSetting = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) =>
      settingsApi.set(key, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useDeleteSetting = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (key: string) => settingsApi.delete(key),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

// ── Hooks ──

export const useInstallHook = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (tool: string) => hooksApi.install(tool),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hooksStatus"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};

export const useUninstallHook = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (tool: string) => hooksApi.uninstall(tool),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hooksStatus"] });
    },
    onError: (error: unknown) => {
      toast.error(getErrorMessage(error));
    },
  });
};
