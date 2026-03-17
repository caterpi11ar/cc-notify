import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  Loader2,
  Globe,
  Clock,
  Gauge,
  Volume2,
  Mic,
  Plug,
  Info,
  ExternalLink,
  Download,
  RefreshCw,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
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
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  useSettingsQuery,
  useHooksStatusQuery,
  useSetSetting,
  useInstallHook,
  useUninstallHook,
} from "@/lib/query";

const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh", label: "中文" },
  { value: "ja", label: "日本語" },
] as const;

const DAY_KEYS = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"] as const;

const HOOK_TOOLS = [
  { key: "claude", label: "Claude Code" },
  { key: "codex", label: "Codex" },
  { key: "gemini", label: "Gemini CLI" },
] as const;

export function SettingsPage() {
  const { t, i18n } = useTranslation();

  const { data: settings, isLoading: settingsLoading } = useSettingsQuery();
  const { data: hooksStatus, isLoading: hooksLoading } =
    useHooksStatusQuery();
  const setSetting = useSetSetting();
  const installHook = useInstallHook();
  const uninstallHook = useUninstallHook();

  // Local state mirrors for responsive UI (avoids mutation on every keystroke)
  const [language, setLanguage] = useState(i18n.language);
  const [historyRetention, setHistoryRetention] = useState("30");
  const [quietHoursEnabled, setQuietHoursEnabled] = useState(false);
  const [quietHoursStart, setQuietHoursStart] = useState("22:00");
  const [quietHoursEnd, setQuietHoursEnd] = useState("08:00");
  const [quietHoursDays, setQuietHoursDays] = useState<string[]>([]);
  const [maxPerMinute, setMaxPerMinute] = useState("10");
  const [cooldownSeconds, setCooldownSeconds] = useState("5");
  const [soundEnabled, setSoundEnabled] = useState(true);
  const [volume, setVolume] = useState("80");
  const [voiceEnabled, setVoiceEnabled] = useState(false);
  const [voiceName, setVoiceName] = useState("");

  // Sync local state from fetched settings
  useEffect(() => {
    if (!settings) return;
    if (settings["language"]) {
      setLanguage(settings["language"]);
    }
    if (settings["history_retention_days"]) {
      setHistoryRetention(settings["history_retention_days"]);
    }
    setQuietHoursEnabled(settings["quiet_hours_enabled"] === "true");
    if (settings["quiet_hours_start"]) {
      setQuietHoursStart(settings["quiet_hours_start"]);
    }
    if (settings["quiet_hours_end"]) {
      setQuietHoursEnd(settings["quiet_hours_end"]);
    }
    if (settings["quiet_hours_days"]) {
      const raw = settings["quiet_hours_days"];
      // Support both JSON array and comma-separated formats
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          setQuietHoursDays(parsed);
        }
      } catch {
        setQuietHoursDays(raw.split(",").filter(Boolean));
      }
    }
    if (settings["rate_limit_max_per_minute"]) {
      setMaxPerMinute(settings["rate_limit_max_per_minute"]);
    }
    if (settings["rate_limit_cooldown_seconds"]) {
      setCooldownSeconds(settings["rate_limit_cooldown_seconds"]);
    }
    setSoundEnabled(settings["sound_enabled"] === "true");
    if (settings["sound_volume"]) {
      setVolume(settings["sound_volume"]);
    }
    setVoiceEnabled(settings["voice_enabled"] === "true");
    if (settings["voice_name"]) {
      setVoiceName(settings["voice_name"]);
    }
  }, [settings]);

  const saveSetting = useCallback(
    (key: string, value: string) => {
      setSetting.mutate(
        { key, value },
        {
          onSuccess: () => {
            toast.success(t("settings.saveSuccess"));
          },
        },
      );
    },
    [setSetting, t],
  );

  // --- General ---
  const handleLanguageChange = (value: string) => {
    setLanguage(value);
    i18n.changeLanguage(value);
    try {
      window.localStorage.setItem("language", value);
    } catch {
      // ignore
    }
    saveSetting("language", value);
  };

  const handleHistoryRetentionBlur = () => {
    saveSetting("history_retention_days", historyRetention);
  };

  // --- Quiet Hours ---
  const handleQuietHoursToggle = (checked: boolean) => {
    setQuietHoursEnabled(checked);
    saveSetting("quiet_hours_enabled", String(checked));
  };

  const handleQuietHoursStartBlur = () => {
    saveSetting("quiet_hours_start", quietHoursStart);
  };

  const handleQuietHoursEndBlur = () => {
    saveSetting("quiet_hours_end", quietHoursEnd);
  };

  const handleQuietHoursDayToggle = (day: string) => {
    const newDays = quietHoursDays.includes(day)
      ? quietHoursDays.filter((d) => d !== day)
      : [...quietHoursDays, day];
    setQuietHoursDays(newDays);
    saveSetting("quiet_hours_days", JSON.stringify(newDays));
  };

  // --- Rate Limiting ---
  const handleMaxPerMinuteBlur = () => {
    saveSetting("rate_limit_max_per_minute", maxPerMinute);
  };

  const handleCooldownBlur = () => {
    saveSetting("rate_limit_cooldown_seconds", cooldownSeconds);
  };

  // --- Sound ---
  const handleSoundToggle = (checked: boolean) => {
    setSoundEnabled(checked);
    saveSetting("sound_enabled", String(checked));
  };

  const handleVolumeBlur = () => {
    const clamped = Math.max(0, Math.min(100, Number(volume) || 0));
    setVolume(String(clamped));
    saveSetting("sound_volume", String(clamped));
  };

  // --- Voice ---
  const handleVoiceToggle = (checked: boolean) => {
    setVoiceEnabled(checked);
    saveSetting("voice_enabled", String(checked));
  };

  const handleVoiceNameBlur = () => {
    saveSetting("voice_name", voiceName);
  };

  // --- Hooks ---
  const handleInstallHook = (tool: string) => {
    installHook.mutate(tool, {
      onSuccess: () => {
        toast.success(t("settings.installSuccess"));
      },
    });
  };

  const handleUninstallHook = (tool: string) => {
    uninstallHook.mutate(tool, {
      onSuccess: () => {
        toast.success(t("settings.uninstallSuccess"));
      },
    });
  };

  // --- About ---
  const [appVersion, setAppVersion] = useState("");
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "up-to-date" | "available" | "error"
  >("idle");
  const [latestVersion, setLatestVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  const handleCheckUpdate = async () => {
    setUpdateStatus("checking");
    try {
      const res = await fetch(
        "https://api.github.com/repos/caterpi11ar/cc-notify/releases/latest",
      );
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      const remote = (data.tag_name as string).replace(/^v/, "");
      setLatestVersion(remote);

      const isNewer = (a: string, b: string) => {
        const pa = a.split(".").map(Number);
        const pb = b.split(".").map(Number);
        for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
          const va = pa[i] ?? 0;
          const vb = pb[i] ?? 0;
          if (va > vb) return true;
          if (va < vb) return false;
        }
        return false;
      };

      setUpdateStatus(isNewer(remote, appVersion) ? "available" : "up-to-date");
    } catch {
      setUpdateStatus("error");
      toast.error(t("settings.checkFailed"));
    }
  };

  const handleOpenRelease = () => {
    openUrl(
      `https://github.com/caterpi11ar/cc-notify/releases/tag/v${latestVersion}`,
    );
  };

  const handleOpenGithub = () => {
    openUrl("https://github.com/caterpi11ar/cc-notify");
  };

  if (settingsLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">{t("settings.title")}</h2>

      <ScrollArea className="h-[calc(100vh-200px)]">
        <div className="space-y-4 pr-3">
          {/* General Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-2">
                <Globe className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base">
                  {t("settings.general")}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <Label>{t("settings.language")}</Label>
                <Select value={language} onValueChange={handleLanguageChange}>
                  <SelectTrigger className="w-[180px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {LANGUAGES.map((lang) => (
                      <SelectItem key={lang.value} value={lang.value}>
                        {lang.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center justify-between">
                <Label>{t("settings.historyRetention")}</Label>
                <Input
                  type="number"
                  min="1"
                  max="365"
                  value={historyRetention}
                  onChange={(e) => setHistoryRetention(e.target.value)}
                  onBlur={handleHistoryRetentionBlur}
                  className="w-[180px]"
                />
              </div>
            </CardContent>
          </Card>

          {/* Quiet Hours Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Clock className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-base">
                    {t("settings.quietHours")}
                  </CardTitle>
                </div>
                <Switch
                  checked={quietHoursEnabled}
                  onCheckedChange={handleQuietHoursToggle}
                />
              </div>
              <CardDescription>{t("settings.quietHoursDesc")}</CardDescription>
            </CardHeader>
            {quietHoursEnabled && (
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label>{t("settings.quietHoursStart")}</Label>
                  <Input
                    type="time"
                    value={quietHoursStart}
                    onChange={(e) => setQuietHoursStart(e.target.value)}
                    onBlur={handleQuietHoursStartBlur}
                    className="w-[180px]"
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label>{t("settings.quietHoursEnd")}</Label>
                  <Input
                    type="time"
                    value={quietHoursEnd}
                    onChange={(e) => setQuietHoursEnd(e.target.value)}
                    onBlur={handleQuietHoursEndBlur}
                    className="w-[180px]"
                  />
                </div>
                <div className="space-y-2">
                  <Label>{t("settings.quietHoursDays")}</Label>
                  <div className="flex flex-wrap gap-3">
                    {DAY_KEYS.map((day) => (
                      <label
                        key={day}
                        className="flex items-center gap-1.5 text-sm cursor-pointer"
                      >
                        <Checkbox
                          checked={quietHoursDays.includes(day)}
                          onCheckedChange={() =>
                            handleQuietHoursDayToggle(day)
                          }
                        />
                        <span>{t(`settings.days.${day}`)}</span>
                      </label>
                    ))}
                  </div>
                </div>
              </CardContent>
            )}
          </Card>

          {/* Rate Limiting Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-2">
                <Gauge className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base">
                  {t("settings.rateLimit")}
                </CardTitle>
              </div>
              <CardDescription>{t("settings.rateLimitDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <Label>{t("settings.maxPerMinute")}</Label>
                <Input
                  type="number"
                  min="1"
                  max="1000"
                  value={maxPerMinute}
                  onChange={(e) => setMaxPerMinute(e.target.value)}
                  onBlur={handleMaxPerMinuteBlur}
                  className="w-[180px]"
                />
              </div>
              <div className="flex items-center justify-between">
                <Label>{t("settings.cooldownSeconds")}</Label>
                <Input
                  type="number"
                  min="0"
                  max="3600"
                  value={cooldownSeconds}
                  onChange={(e) => setCooldownSeconds(e.target.value)}
                  onBlur={handleCooldownBlur}
                  className="w-[180px]"
                />
              </div>
            </CardContent>
          </Card>

          {/* Sound Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Volume2 className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-base">
                    {t("settings.sound")}
                  </CardTitle>
                </div>
                <Switch
                  checked={soundEnabled}
                  onCheckedChange={handleSoundToggle}
                />
              </div>
              <CardDescription>{t("settings.soundDesc")}</CardDescription>
            </CardHeader>
            {soundEnabled && (
              <CardContent>
                <div className="flex items-center justify-between">
                  <Label>{t("settings.volume")}</Label>
                  <Input
                    type="number"
                    min="0"
                    max="100"
                    value={volume}
                    onChange={(e) => setVolume(e.target.value)}
                    onBlur={handleVolumeBlur}
                    className="w-[180px]"
                  />
                </div>
              </CardContent>
            )}
          </Card>

          {/* Voice Section (macOS only) */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Mic className="h-4 w-4 text-muted-foreground" />
                  <CardTitle className="text-base">
                    {t("settings.voice")}
                  </CardTitle>
                </div>
                <Switch
                  checked={voiceEnabled}
                  onCheckedChange={handleVoiceToggle}
                />
              </div>
              <CardDescription>{t("settings.voiceDesc")}</CardDescription>
            </CardHeader>
            {voiceEnabled && (
              <CardContent>
                <div className="flex items-center justify-between">
                  <Label>{t("settings.voiceName")}</Label>
                  <Input
                    value={voiceName}
                    onChange={(e) => setVoiceName(e.target.value)}
                    onBlur={handleVoiceNameBlur}
                    placeholder="Samantha"
                    className="w-[180px]"
                  />
                </div>
              </CardContent>
            )}
          </Card>

          {/* Hooks Integration Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-2">
                <Plug className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base">
                  {t("settings.hooks")}
                </CardTitle>
              </div>
              <CardDescription>{t("settings.hooksDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {hooksLoading ? (
                <div className="flex items-center justify-center py-4">
                  <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                </div>
              ) : (
                HOOK_TOOLS.map((tool) => {
                  const isInstalled =
                    hooksStatus?.[
                      tool.key as keyof typeof hooksStatus
                    ] ?? false;
                  const isInstalling =
                    installHook.isPending &&
                    installHook.variables === tool.key;
                  const isUninstalling =
                    uninstallHook.isPending &&
                    uninstallHook.variables === tool.key;
                  return (
                    <div
                      key={tool.key}
                      className="flex items-center justify-between rounded-lg border p-3"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-medium">
                          {tool.label}
                        </span>
                        {isInstalled ? (
                          <Badge className="bg-green-500/15 text-green-600 border-green-500/20 hover:bg-green-500/15 flex items-center gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            {t("settings.hooksInstalled")}
                          </Badge>
                        ) : (
                          <Badge
                            variant="secondary"
                            className="flex items-center gap-1"
                          >
                            <XCircle className="h-3 w-3" />
                            {t("settings.hooksNotInstalled")}
                          </Badge>
                        )}
                      </div>
                      {isInstalled ? (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleUninstallHook(tool.key)}
                          disabled={isUninstalling}
                        >
                          {isUninstalling && (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          )}
                          {t("common.uninstall")}
                        </Button>
                      ) : (
                        <Button
                          size="sm"
                          onClick={() => handleInstallHook(tool.key)}
                          disabled={isInstalling}
                        >
                          {isInstalling && (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          )}
                          {t("common.install")}
                        </Button>
                      )}
                    </div>
                  );
                })
              )}
            </CardContent>
          </Card>

          {/* About Section */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center gap-2">
                <Info className="h-4 w-4 text-muted-foreground" />
                <CardTitle className="text-base">
                  {t("settings.about")}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">
                  CC Notify{appVersion ? ` v${appVersion}` : ""}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCheckUpdate}
                  disabled={updateStatus === "checking"}
                >
                  {updateStatus === "checking" ? (
                    <RefreshCw className="h-3 w-3 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3 w-3" />
                  )}
                  {updateStatus === "checking"
                    ? t("settings.checking")
                    : t("settings.checkUpdate")}
                </Button>
              </div>

              {updateStatus === "up-to-date" && (
                <div className="flex items-center gap-2 text-sm text-green-600">
                  <CheckCircle2 className="h-4 w-4" />
                  <span>{t("settings.upToDate")}</span>
                </div>
              )}

              {updateStatus === "available" && (
                <div className="flex items-center justify-between rounded-lg border border-blue-500/30 bg-blue-500/5 p-3">
                  <span className="text-sm">
                    {t("settings.newVersion", { version: latestVersion })}
                  </span>
                  <Button size="sm" onClick={handleOpenRelease}>
                    <Download className="h-3 w-3" />
                    {t("settings.download")}
                  </Button>
                </div>
              )}

              <Button
                variant="ghost"
                size="sm"
                className="gap-1.5"
                onClick={handleOpenGithub}
              >
                <ExternalLink className="h-3 w-3" />
                {t("settings.viewOnGithub")}
              </Button>
            </CardContent>
          </Card>
        </div>
      </ScrollArea>
    </div>
  );
}
