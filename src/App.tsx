import { useEffect, useState, type ReactNode } from "react";
import {
  Activity,
  BarChart3,
  Clipboard,
  Database,
  KeyRound,
  Play,
  RefreshCcw,
  Square,
} from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  gatewayApi,
  type ApiKeyStatus,
  type GatewayLog,
  type GatewayStatus,
  type Provider,
} from "@/lib/api";

function App() {
  const [activeTab, setActiveTab] = useState("dashboard");
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [apiKey, setApiKey] = useState<string>("");
  const [providers, setProviders] = useState<Provider[]>([]);
  const [logs, setLogs] = useState<GatewayLog[]>([]);

  async function refresh() {
    const [nextStatus, nextKeyStatus, nextKey, nextProviders, nextLogs] = await Promise.all([
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

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,#e7f5ef,transparent_30%),linear-gradient(135deg,#f8faf7,#eef3f5_45%,#fff7ed)] text-slate-950">
      <div className="mx-auto flex h-screen max-w-7xl flex-col px-4 py-4">
        <header className="mb-4 flex flex-wrap items-center justify-between gap-3">
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

        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="flex min-h-0 flex-1 flex-col"
        >
          <TabsList className="grid h-auto w-full grid-cols-2 gap-1 bg-white/70 p-1 shadow-sm">
            <Tab value="dashboard" icon={<Activity />} label="Dashboard" />
            <Tab value="usage" icon={<BarChart3 />} label="Usage" />
          </TabsList>

          <div className="mt-4 min-h-0 flex-1 overflow-auto">
            <TabsContent value="dashboard" className="mt-0">
              <Dashboard
                status={status}
                apiKey={apiKey}
                apiKeyStatus={apiKeyStatus}
                providers={providers}
                logs={logs}
                onCopy={copy}
                onStart={() => run(() => gatewayApi.start(), "Gateway started")}
                onStop={() => run(() => gatewayApi.stop(), "Gateway stopped")}
                onRestart={() => run(() => gatewayApi.restart(), "Gateway restarted")}
                onRegenerateKey={() =>
                  run(() => gatewayApi.regenerateApiKey(), "Local API key regenerated")
                }
              />
            </TabsContent>

            <TabsContent value="usage" className="mt-0">
              <UsagePage
                logs={logs}
                onRefresh={() => refresh()}
                onClear={() => run(() => gatewayApi.clearLogs(), "Usage cleared")}
              />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </main>
  );
}

function Dashboard(props: {
  status: GatewayStatus | null;
  apiKey: string;
  apiKeyStatus: ApiKeyStatus | null;
  providers: Provider[];
  logs: GatewayLog[];
  onCopy: (value: string) => void;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  onRegenerateKey: () => void;
}) {
  const { status } = props;
  return (
    <div className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
      <section className="grid gap-4 sm:grid-cols-2">
        <Metric title="Gateway" value={status?.running ? "Running" : "Stopped"} />
        <Metric title="Upstream" value={props.providers[0]?.enabled ? "Ready" : "From cc switch"} />
        <CopyBlock title="OpenAI Base URL" value={status?.openai_base_url ?? ""} onCopy={props.onCopy} />
        <CopyBlock title="Anthropic Base URL" value={status?.anthropic_base_url ?? ""} onCopy={props.onCopy} />
        <CopyBlock
          title="Local API Key"
          value={props.apiKey || props.apiKeyStatus?.preview || "Generate a key"}
          onCopy={props.onCopy}
        />
      </section>
      <Card className="rounded-lg bg-white/85">
        <CardHeader>
          <CardTitle>Gateway Control</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-3 gap-2">
            <Button onClick={props.onStart} disabled={status?.running}>
              <Play />
              Start
            </Button>
            <Button variant="outline" onClick={props.onStop} disabled={!status?.running}>
              <Square />
              Stop
            </Button>
            <Button variant="outline" onClick={props.onRestart}>
              <RefreshCcw />
              Restart
            </Button>
          </div>
          <Button variant="secondary" className="w-full" onClick={props.onRegenerateKey}>
            <KeyRound />
            Regenerate Local Key
          </Button>
          <RecentFailures logs={props.logs} />
        </CardContent>
      </Card>
    </div>
  );
}

function UsagePage(props: { logs: GatewayLog[]; onRefresh: () => void; onClear: () => void }) {
  const stats = buildUsageStats(props.logs);

  return (
    <div className="grid gap-4">
      <section className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        <Metric title="Requests" value={String(stats.requests)} />
        <Metric title="Success Rate" value={`${stats.successRate}%`} />
        <Metric title="Input Tokens" value={formatCount(stats.inputTokens)} />
        <Metric title="Output Tokens" value={formatCount(stats.outputTokens)} />
        <Metric title="Total Tokens" value={formatCount(stats.totalTokens)} />
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="rounded-lg bg-white/85">
          <CardHeader>
            <CardTitle>Upstream Usage</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {stats.byProvider.length === 0 ? (
              <p className="text-sm text-slate-500">No usage recorded.</p>
            ) : (
              stats.byProvider.map((item) => (
                <UsageRow key={item.name} item={item} />
              ))
            )}
          </CardContent>
        </Card>

        <Card className="rounded-lg bg-white/85">
          <CardHeader>
            <CardTitle>Model Usage</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {stats.byModel.length === 0 ? (
              <p className="text-sm text-slate-500">No usage recorded.</p>
            ) : (
              stats.byModel.map((item) => (
                <UsageRow key={item.name} item={item} />
              ))
            )}
          </CardContent>
        </Card>
      </div>

      <Card className="rounded-lg bg-white/85">
        <CardHeader className="gap-3 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
          <div>
            <CardTitle>Usage Events</CardTitle>
            <CardDescription>
              Forwarded requests, upstream status, latency, and token usage.
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={props.onRefresh}>
              Refresh
            </Button>
            <Button variant="destructive" size="sm" onClick={props.onClear}>
              Clear
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {props.logs.length === 0 ? (
            <EmptyState title="No usage yet" description="Requests will appear here after the gateway forwards them." />
          ) : (
            <ScrollArea className="h-[min(58vh,620px)] pr-3">
              <div className="space-y-3">
                {props.logs.map((log) => <UsageEvent key={log.id} log={log} />)}
              </div>
            </ScrollArea>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

interface UsageAggregate {
  name: string;
  requests: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  cacheTokens: number;
}

function UsageRow(props: { item: UsageAggregate }) {
  return (
    <div className="rounded-lg border bg-white/70 px-3 py-2">
      <div className="flex items-center justify-between gap-3">
        <p className="truncate font-semibold">{props.item.name}</p>
        <Badge>
          {props.item.requests} req
        </Badge>
      </div>
      <div className="mt-2 grid grid-cols-4 gap-2 text-xs text-slate-600">
        <span>in {formatCount(props.item.inputTokens)}</span>
        <span>out {formatCount(props.item.outputTokens)}</span>
        <span>total {formatCount(props.item.totalTokens)}</span>
        <span>cache {formatCount(props.item.cacheTokens)}</span>
      </div>
    </div>
  );
}

function UsageEvent(props: { log: GatewayLog }) {
  const log = props.log;
  const model = log.resolved_model || log.requested_model || "-";
  const provider = log.resolved_provider || "No upstream";
  const total = log.total_tokens ?? log.token_estimate;
  const cache = (log.cache_creation_input_tokens ?? 0) + (log.cache_read_input_tokens ?? 0);
  const ok = log.status_code >= 200 && log.status_code < 300;

  return (
    <div className="rounded-lg border bg-white/75 p-3">
      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="min-w-0 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-mono text-xs text-slate-500">
              {new Date(log.time * 1000).toLocaleTimeString()}
            </span>
            <Badge variant="secondary" className="font-mono">
              {log.endpoint}
            </Badge>
            <StatusCode value={log.status_code} ok={ok} />
          </div>
          <p className="break-words text-sm font-semibold text-slate-950">
            {model}
          </p>
          <p className="text-xs text-slate-500">
            {provider} · {log.protocol_in}
            {log.protocol_out ? ` -> ${log.protocol_out}` : ""} · {log.latency_ms}ms
          </p>
        </div>

        <div className="grid grid-cols-2 gap-2 text-xs text-slate-600 sm:grid-cols-4 xl:text-right">
          <TokenStat label="Input" value={log.input_tokens} />
          <TokenStat label="Output" value={log.output_tokens} />
          <TokenStat label="Total" value={total} />
          <TokenStat label="Cache" value={cache} />
        </div>
      </div>
      {log.error_message ? (
        <p className="mt-3 rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
          {log.error_message}
        </p>
      ) : null}
    </div>
  );
}

function StatusCode(props: { value: number; ok: boolean }) {
  return (
    <Badge
      variant={props.ok ? "outline" : "destructive"}
      className={props.ok ? "border-emerald-200 bg-emerald-50 text-emerald-800" : undefined}
    >
      {props.value}
    </Badge>
  );
}

function TokenStat(props: { label: string; value: number | null | undefined }) {
  return (
    <div className="rounded-md bg-slate-50 px-2 py-1.5">
      <p className="uppercase tracking-[0.12em] text-slate-400">{props.label}</p>
      <p className="mt-1 font-mono text-sm text-slate-900">{formatMaybeCount(props.value)}</p>
    </div>
  );
}

function EmptyState(props: { title: string; description: string }) {
  return (
    <div className="rounded-lg border border-dashed bg-slate-50/70 px-4 py-8 text-center">
      <p className="text-sm font-semibold text-slate-900">{props.title}</p>
      <p className="mt-1 text-sm text-slate-500">{props.description}</p>
    </div>
  );
}

function buildUsageStats(logs: GatewayLog[]) {
  const successful = logs.filter((log) => log.status_code >= 200 && log.status_code < 300);
  const byProvider = new Map<string, UsageAggregate>();
  const byModel = new Map<string, UsageAggregate>();

  let inputTokens = 0;
  let outputTokens = 0;
  let totalTokens = 0;

  for (const log of logs) {
    const input = log.input_tokens ?? 0;
    const output = log.output_tokens ?? 0;
    const total = log.total_tokens ?? log.token_estimate ?? input + output;
    const cache =
      (log.cache_creation_input_tokens ?? 0) + (log.cache_read_input_tokens ?? 0);

    inputTokens += input;
    outputTokens += output;
    totalTokens += total;

    addUsageAggregate(byProvider, log.resolved_provider || "Unresolved", {
      input,
      output,
      total,
      cache,
    });
    addUsageAggregate(byModel, log.resolved_model || log.requested_model || "Unknown", {
      input,
      output,
      total,
      cache,
    });
  }

  return {
    requests: logs.length,
    successRate: logs.length === 0 ? 0 : Math.round((successful.length / logs.length) * 100),
    inputTokens,
    outputTokens,
    totalTokens,
    byProvider: [...byProvider.values()].sort((a, b) => b.totalTokens - a.totalTokens),
    byModel: [...byModel.values()].sort((a, b) => b.totalTokens - a.totalTokens),
  };
}

function addUsageAggregate(
  map: Map<string, UsageAggregate>,
  name: string,
  usage: { input: number; output: number; total: number; cache: number },
) {
  const item =
    map.get(name) ??
    {
      name,
      requests: 0,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      cacheTokens: 0,
    };

  item.requests += 1;
  item.inputTokens += usage.input;
  item.outputTokens += usage.output;
  item.totalTokens += usage.total;
  item.cacheTokens += usage.cache;
  map.set(name, item);
}

function formatCount(value: number) {
  return new Intl.NumberFormat().format(value);
}

function formatMaybeCount(value: number | null | undefined) {
  return value == null ? "-" : formatCount(value);
}

function Metric(props: { title: string; value: string }) {
  return (
    <Card className="rounded-lg bg-white/85">
      <CardContent className="p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">
          {props.title}
        </p>
        <p className="mt-2 text-2xl font-black">{props.value}</p>
      </CardContent>
    </Card>
  );
}

function CopyBlock(props: {
  title: string;
  value: string;
  onCopy: (value: string) => void;
}) {
  return (
    <Card className="rounded-lg bg-white/85">
      <CardContent className="flex items-center justify-between gap-3 p-4">
        <div className="min-w-0">
          <p className="text-xs font-semibold uppercase tracking-[0.16em] text-slate-500">
            {props.title}
          </p>
          <p className="truncate font-mono text-sm">{props.value || "-"}</p>
        </div>
        <Button variant="outline" size="icon" onClick={() => props.onCopy(props.value)}>
          <Clipboard />
        </Button>
      </CardContent>
    </Card>
  );
}

function RecentFailures(props: { logs: GatewayLog[] }) {
  const failures = props.logs.filter((log) => log.status_code >= 400).slice(0, 4);
  return (
    <div className="rounded-lg border bg-white/70 p-3">
      <div className="mb-2 flex items-center gap-2 text-sm font-semibold">
        <Database className="h-4 w-4" />
        Recent failures
      </div>
      {failures.length === 0 ? (
        <p className="text-sm text-slate-500">No failures recorded.</p>
      ) : (
        <div className="space-y-2">
          {failures.map((log) => (
            <p key={log.id} className="truncate text-sm text-red-700">
              {log.endpoint} · {log.error_message || log.status_code}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}

function StatusPill(props: { running: boolean }) {
  return (
    <span
      className={`rounded-full px-3 py-1 text-sm font-semibold ${
        props.running ? "bg-emerald-600 text-white" : "bg-slate-200 text-slate-700"
      }`}
    >
      {props.running ? "Running" : "Stopped"}
    </span>
  );
}

function Tab(props: { value: string; icon: ReactNode; label: string }) {
  return (
    <TabsTrigger value={props.value} className="gap-1.5">
      {props.icon}
      {props.label}
    </TabsTrigger>
  );
}

function messageOf(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "Unexpected error";
}

export default App;
