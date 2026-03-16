import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Trash2, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import {
  useHistoryQuery,
  useChannelsQuery,
  useEventTypesQuery,
  useClearHistory,
} from "@/lib/query";

export function HistoryPage() {
  const { t } = useTranslation();
  const [limit, setLimit] = useState(50);
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  const { data: history, isLoading: historyLoading } = useHistoryQuery(
    limit,
    0,
  );
  const { data: channels } = useChannelsQuery();
  const { data: eventTypes } = useEventTypesQuery();
  const clearHistory = useClearHistory();

  const channelMap = useMemo(() => {
    const map = new Map<string, string>();
    channels?.forEach((c) => map.set(c.id, c.name));
    return map;
  }, [channels]);

  const eventTypeMap = useMemo(() => {
    const map = new Map<string, string>();
    eventTypes?.forEach((e) => map.set(e.id, e.name));
    return map;
  }, [eventTypes]);

  const handleClear = () => {
    setShowClearConfirm(true);
  };

  const confirmClear = useCallback(() => {
    clearHistory.mutate(undefined, {
      onSuccess: () => {
        toast.success(t("history.clearSuccess"));
      },
    });
    setShowClearConfirm(false);
  }, [clearHistory, t]);

  const handleLoadMore = () => {
    setLimit((prev) => prev + 50);
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "sent":
        return (
          <Badge className="bg-green-500/15 text-green-600 border-green-500/20 hover:bg-green-500/15">
            {t("history.status.sent")}
          </Badge>
        );
      case "failed":
        return (
          <Badge className="bg-red-500/15 text-red-600 border-red-500/20 hover:bg-red-500/15">
            {t("history.status.failed")}
          </Badge>
        );
      case "skipped":
        return (
          <Badge className="bg-yellow-500/15 text-yellow-600 border-yellow-500/20 hover:bg-yellow-500/15">
            {t("history.status.skipped")}
          </Badge>
        );
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  const formatTimestamp = (ts: number) => {
    return new Date(ts * 1000).toLocaleString();
  };

  if (historyLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("history.title")}</h2>
        {history && history.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleClear}
            disabled={clearHistory.isPending}
            className="text-destructive hover:text-destructive"
          >
            {clearHistory.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Trash2 className="h-4 w-4" />
            )}
            {t("history.clear")}
          </Button>
        )}
      </div>

      {!history?.length ? (
        <p className="text-muted-foreground py-8 text-center">
          {t("history.empty")}
        </p>
      ) : (
        <>
          <ScrollArea className="h-[calc(100vh-240px)]">
            <div className="rounded-md border">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b bg-muted/50">
                    <th className="px-3 py-2 text-left font-medium">
                      {t("history.columns.timestamp")}
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      {t("history.columns.event")}
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      {t("history.columns.channel")}
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      {t("history.columns.status")}
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      {t("history.columns.message")}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {history.map((entry) => (
                    <tr key={entry.id} className="border-b last:border-b-0">
                      <td className="px-3 py-2 text-xs text-muted-foreground whitespace-nowrap">
                        {formatTimestamp(entry.created_at)}
                      </td>
                      <td className="px-3 py-2 text-xs">
                        {eventTypeMap.get(entry.event_type_id) ??
                          entry.event_type_id}
                      </td>
                      <td className="px-3 py-2 text-xs">
                        {channelMap.get(entry.channel_id) ?? entry.channel_id}
                      </td>
                      <td className="px-3 py-2">
                        {getStatusBadge(entry.status)}
                      </td>
                      <td className="px-3 py-2 text-xs max-w-[200px] truncate">
                        {entry.error_message ?? entry.message_body}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </ScrollArea>
          {history.length >= limit && (
            <div className="flex justify-center">
              <Button variant="outline" size="sm" onClick={handleLoadMore}>
                {t("history.loadMore")}
              </Button>
            </div>
          )}
        </>
      )}

      {/* Clear Confirmation */}
      <ConfirmDialog
        open={showClearConfirm}
        onOpenChange={setShowClearConfirm}
        description={t("history.clearConfirm")}
        onConfirm={confirmClear}
        variant="destructive"
      />
    </div>
  );
}
