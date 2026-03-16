import { useQuery } from "@tanstack/react-query";
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

export const useChannelsQuery = () =>
  useQuery({
    queryKey: ["channels"],
    queryFn: () => channelsApi.getAll(),
  });

export const useEventTypesQuery = () =>
  useQuery({
    queryKey: ["eventTypes"],
    queryFn: () => eventTypesApi.getAll(),
  });

export const useRulesQuery = () =>
  useQuery({
    queryKey: ["rules"],
    queryFn: () => rulesApi.getAll(),
  });

export const useRoutingQuery = () =>
  useQuery({
    queryKey: ["routing"],
    queryFn: () => routingApi.getAll(),
  });

export const useTemplatesQuery = () =>
  useQuery({
    queryKey: ["templates"],
    queryFn: () => templatesApi.getAll(),
  });

export const useHistoryQuery = (limit = 50, offset = 0) =>
  useQuery({
    queryKey: ["history", limit, offset],
    queryFn: () => historyApi.getAll(limit, offset),
  });

export const useSettingsQuery = () =>
  useQuery({
    queryKey: ["settings"],
    queryFn: () => settingsApi.getAll(),
  });

export const useHooksStatusQuery = () =>
  useQuery({
    queryKey: ["hooksStatus"],
    queryFn: () => hooksApi.getStatus(),
  });
