import { useEffect, useState } from "react";
import {
  Activity,
  Clipboard,
  KeyRound,
  Play,
  RefreshCcw,
  Square,
} from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  gatewayApi,
  type ApiKeyStatus,
  type GatewayLog,
  type GatewayStatus,
  type Provider,
} from "@/lib/api";

function App() {
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [providers, setProviders] = useState<Provider[]>([]);
  const [logs, setLogs] = useState<GatewayLog[]>([]);

  async function refresh() {
    const [nextStatus, nextKeyStatus, nextKey, nextProviders, nextLogs] =
      await Promise.all([
        gatewayApi.getStatus(),
        gatewayApi.getApiKeyStatus(),
        gatewayApi.getApiKeyOnce(),
        gatewayApi.getProviders(),
        gatewayApi.getLogs(100),
      ]);
    setStatus(nextStatus);
    setApiKeyStatus(nextKeyStatus);
    setApiKey(nextKey ?? "");
    setProviders(nextProviders);
    setLogs(nextLogs);
  }

  useEffect(() => {
    refresh().catch((error) => toast.error(messageOf(error)));
  }, []);

  async function run(action: () => Promise<unknown>, success?: string) {
    try {
      await action();
      await refresh();
      if (success) toast.success(success);
    } catch (error) {
      toast.error(messageOf(error));
    }
  }

  async function copy(value: string) {
    await navigator.clipboard.writeText(value);
    toast.success("Copied");
  }

  const stats = buildUsageStats(logs);

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,#e7f5ef,transparent_30%),linear-gradient(135deg,#f8faf7,#eef3f5_45%,#fff7ed)] text-slate-950">
      <div className="mx-auto flex min-h-screen max-w-7xl flex-col gap-4 px-4 py-4">
        <header className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.22em] text-emerald-700">
              Local AI Gateway
            </p>
            <h1 className="text-2xl font-black tracking-normal">
              Transparent local gateway
            </h1>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusPill running={status?.running ?? false} />
            <Button variant="outline" size="sm" onClick={() => refresh()}>
              <RefreshCcw />
              Refresh
            </Button>
          </div>
        </header>

        <section className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
          <Card className="rounded-lg bg-white/85">
            <CardHeader>
              <CardTitle>Gateway</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 sm:grid-cols-2">
              <Metric title="Status" value={status?.running ? "Running" : "Stopped"} />
              <Metric title="Upstream" value={providers[0]?.enabled ? "Ready" : "Not set"} />
              <CopyBlock
                title="OpenAI Base URL"
                value={status?.openai_base_url ?? ""}
                onCopy={copy}
              />
              <CopyBlock
                title="Anthropic Base URL"
                value={status?.anthropic_base_url ?? ""}
                onCopy={copy}
              />
              <CopyBlock
                title="Local API Key"
                value={apiKey || apiKeyStatus?.preview || "Generate a key"}
                onCopy={copy}
              />
              <div className="grid gap-2">
                <Button onClick={() => run(() => gatewayApi.start(), "Gateway started")}>
                  <Play />
                  Start
                </Button>
                <Button
                  variant="outline"
                  onClick={() => run(() => gatewayApi.stop(), "Gateway stopped")}
                  disabled={!status?.running}
                >
                  <Square />
                  Stop
                </Button>
                <Button
                  variant="outline"
                  onClick={() => run(() => gatewayApi.restart(), "Gateway restarted")}
                >
                  <RefreshCcw />
                  Restart
                </Button>
                <Button
                  variant="secondary"
                  onClick={() =>
                    run(
                      () => gatewayApi.regenerateApiKey(),
                      "Local API key regenerated",
                    )
                  }
                >
                  <KeyRound />
                  Regenerate Key
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card className="rounded-lg bg-white/85">
            <CardHeader>
              <CardTitle>Usage</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
              <Metric title="Requests" value={String(stats.requests)} />
              <Metric title="Success Rate" value={`${stats.successRate}%`} />
              <Metric title="Input Tokens" value={formatCount(stats.inputTokens)} />
              <Metric title="Output Tokens" value={formatCount(stats.outputTokens)} />
              <Metric title="Total Tokens" value={formatCount(stats.totalTokens)} />
            </CardContent>
          </Card>
        </section>

        <section className="grid gap-4 lg:grid-cols-2">
          <Card className="rounded-lg bg-white/85">
            <CardHeader>
              <CardTitle>Providers</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {providers.length === 0 ? (
                <p className="text-sm text-slate-500">No enabled provider.</p>
              ) : (
                providers.map((provider) => (
                  <div
                    key={provider.id}
                    className="flex items-center justify-between rounded-md border border-slate-200 px-3 py-2"
                  >
                    <div>
                      <div className="font-medium">{provider.name}</div>
                      <div className="text-xs text-slate-500">{provider.base_url}</div>
                    </div>
                    <Badge variant={provider.enabled ? "default" : "secondary"}>
                      {provider.protocol}
                    </Badge>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          <Card className="rounded-lg bg-white/85">
            <CardHeader>
              <CardTitle>Recent Logs</CardTitle>
            </CardHeader>
            <CardContent>
              <ScrollArea className="h-[360px] pr-3">
                <div className="space-y-2">
                  {logs.length === 0 ? (
                    <p className="text-sm text-slate-500">No usage recorded.</p>
                  ) : (
                    logs.map((log) => (
                      <div
                        key={log.id}
                        className="rounded-md border border-slate-200 px-3 py-2 text-sm"
                      >
                        <div className="flex items-center justify-between gap-2">
                          <span className="font-medium">{log.endpoint}</span>
                          <span className="text-xs text-slate-500">{log.status_code}</span>
                        </div>
                        <div className="mt-1 text-xs text-slate-500">
                          {log.client_type} · {log.requested_model || "unknown"} ·{" "}
                          {log.latency_ms}ms
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </ScrollArea>
            </CardContent>
          </Card>
        </section>
      </div>
    </main>
  );
}

function StatusPill({ running }: { running: boolean }) {
  return (
    <Badge variant={running ? "default" : "secondary"} className="gap-1">
      <Activity className="h-3.5 w-3.5" />
      {running ? "Running" : "Stopped"}
    </Badge>
  );
}

function Metric({ title, value }: { title: string; value: string }) {
  return (
    <div className="rounded-md border border-slate-200 bg-white px-3 py-2">
      <div className="text-xs uppercase tracking-wide text-slate-500">{title}</div>
      <div className="mt-1 text-lg font-semibold">{value}</div>
    </div>
  );
}

function CopyBlock({
  title,
  value,
  onCopy,
}: {
  title: string;
  value: string;
  onCopy: (value: string) => void;
}) {
  return (
    <button
      type="button"
      className="rounded-md border border-slate-200 bg-white p-3 text-left transition hover:bg-slate-50"
      onClick={() => value && onCopy(value)}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs uppercase tracking-wide text-slate-500">{title}</div>
        <Clipboard className="h-4 w-4 text-slate-400" />
      </div>
      <div className="mt-1 break-all text-sm font-medium">{value || "-"}</div>
    </button>
  );
}

function buildUsageStats(logs: GatewayLog[]) {
  const requests = logs.length;
  const successes = logs.filter((log) => log.status_code >= 200 && log.status_code < 400).length;
  return {
    requests,
    successRate: requests === 0 ? 0 : Math.round((successes / requests) * 100),
    inputTokens: sum(logs, "input_tokens"),
    outputTokens: sum(logs, "output_tokens"),
    totalTokens: sum(logs, "total_tokens"),
  };
}

function sum(logs: GatewayLog[], key: keyof Pick<GatewayLog, "input_tokens" | "output_tokens" | "total_tokens">) {
  return logs.reduce((total, log) => total + (log[key] ?? 0), 0);
}

function formatCount(value: number) {
  return new Intl.NumberFormat("en-US").format(value);
}

function messageOf(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export default App;
