import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Loader2 } from "lucide-react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  useEventTypesQuery,
  useChannelsQuery,
  useRoutingQuery,
  useUpdateEventType,
  useSetRouting,
  useDeleteRouting,
} from "@/lib/query";
import type { EventType, Routing } from "@/types";

const CATEGORY_ORDER = ["claude_hook", "extended", "custom"];

export function EventsPage() {
  const { t } = useTranslation();

  const { data: eventTypes, isLoading: eventsLoading } =
    useEventTypesQuery();
  const { data: channels, isLoading: channelsLoading } = useChannelsQuery();
  const { data: routings, isLoading: routingsLoading } = useRoutingQuery();

  const updateEventType = useUpdateEventType();
  const setRouting = useSetRouting();
  const deleteRouting = useDeleteRouting();

  const isLoading = eventsLoading || channelsLoading || routingsLoading;

  const groupedEvents = useMemo(() => {
    if (!eventTypes) return {};
    const groups: Record<string, EventType[]> = {};
    for (const cat of CATEGORY_ORDER) {
      groups[cat] = [];
    }
    for (const event of eventTypes) {
      const cat = event.category || "custom";
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(event);
    }
    return groups;
  }, [eventTypes]);

  const routingMap = useMemo(() => {
    if (!routings) return new Map<string, Set<string>>();
    const map = new Map<string, Set<string>>();
    for (const r of routings) {
      if (!map.has(r.event_type_id)) {
        map.set(r.event_type_id, new Set());
      }
      if (r.enabled) {
        map.get(r.event_type_id)!.add(r.channel_id);
      }
    }
    return map;
  }, [routings]);

  const handleToggleEvent = (id: string, enabled: boolean) => {
    updateEventType.mutate(
      { id, eventType: { enabled } },
      {
        onSuccess: () => {
          toast.success(t("events.updateSuccess"));
        },
      },
    );
  };

  const handleToggleRouting = (
    eventTypeId: string,
    channelId: string,
    isConnected: boolean,
  ) => {
    if (isConnected) {
      deleteRouting.mutate(
        { eventTypeId, channelId },
        {
          onSuccess: () => {
            toast.success(t("events.routingUpdated"));
          },
        },
      );
    } else {
      const routing: Routing = {
        event_type_id: eventTypeId,
        channel_id: channelId,
        enabled: true,
        priority: 0,
      };
      setRouting.mutate(routing, {
        onSuccess: () => {
          toast.success(t("events.routingUpdated"));
        },
      });
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const hasEvents = eventTypes && eventTypes.length > 0;

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">{t("events.title")}</h2>

      {!hasEvents ? (
        <p className="text-muted-foreground py-8 text-center">
          {t("events.empty")}
        </p>
      ) : (
        <ScrollArea className="h-[calc(100vh-200px)]">
          <div className="space-y-6 pr-3">
            {CATEGORY_ORDER.map((category) => {
              const events = groupedEvents[category];
              if (!events || events.length === 0) return null;
              return (
                <div key={category} className="space-y-3">
                  <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
                    {t(`events.categories.${category}` as const, category)}
                  </h3>
                  {events.map((event) => {
                    const connectedChannels =
                      routingMap.get(event.id) ?? new Set();
                    return (
                      <Card key={event.id}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2 pt-4 px-4">
                          <div className="flex items-center gap-3">
                            <CardTitle className="text-sm font-medium">
                              {event.name}
                            </CardTitle>
                            <Badge
                              variant={
                                event.category === "claude_hook"
                                  ? "default"
                                  : event.category === "extended"
                                    ? "secondary"
                                    : "outline"
                              }
                              className="text-xs"
                            >
                              {t(
                                `events.categories.${event.category}` as const,
                                event.category,
                              )}
                            </Badge>
                            {event.is_builtin && (
                              <Badge variant="outline" className="text-xs">
                                Built-in
                              </Badge>
                            )}
                          </div>
                          <Switch
                            checked={event.enabled}
                            onCheckedChange={(checked) =>
                              handleToggleEvent(event.id, checked)
                            }
                          />
                        </CardHeader>
                        <CardContent className="px-4 pb-3 pt-0">
                          <div className="space-y-2">
                            <p className="text-xs font-medium text-muted-foreground">
                              {t("events.routing")}
                            </p>
                            {channels && channels.length > 0 ? (
                              <div className="flex flex-wrap gap-3">
                                {channels.map((channel) => {
                                  const isConnected = connectedChannels.has(
                                    channel.id,
                                  );
                                  return (
                                    <label
                                      key={channel.id}
                                      className="flex items-center gap-1.5 text-xs cursor-pointer"
                                    >
                                      <Checkbox
                                        checked={isConnected}
                                        onCheckedChange={() =>
                                          handleToggleRouting(
                                            event.id,
                                            channel.id,
                                            isConnected,
                                          )
                                        }
                                        disabled={!event.enabled}
                                      />
                                      <span
                                        className={
                                          event.enabled
                                            ? ""
                                            : "text-muted-foreground"
                                        }
                                      >
                                        {channel.name}
                                      </span>
                                    </label>
                                  );
                                })}
                              </div>
                            ) : (
                              <p className="text-xs text-muted-foreground">
                                {t("events.noChannels")}
                              </p>
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    );
                  })}
                </div>
              );
            })}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
