import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import React from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { createTestQueryClient } from "../utils/testQueryClient";
import { server } from "../msw/server";
import {
  useCreateChannel,
  useUpdateChannel,
  useDeleteChannel,
  useTestChannel,
  useCreateEventType,
  useUpdateEventType,
  useDeleteEventType,
  useCreateRule,
  useUpdateRule,
  useDeleteRule,
  useSetRouting,
  useDeleteRouting,
  useCreateTemplate,
  useUpdateTemplate,
  useDeleteTemplate,
  useClearHistory,
  useSetSetting,
  useDeleteSetting,
  useInstallHook,
  useUninstallHook,
} from "@/lib/query";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

const { toast } = await import("sonner");

function createWrapper() {
  const queryClient = createTestQueryClient();
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  return { wrapper, queryClient };
}

const TAURI_ENDPOINT = "http://tauri.local";

// ── Channels ──

describe("useCreateChannel", () => {
  it("creates a channel and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCreateChannel(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "New Channel",
        channel_type: "webhook",
        config: {},
        enabled: true,
        sort_index: 0,
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["channels"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/create_channel`, () =>
        new HttpResponse("Create failed", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateChannel(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "Fail",
        channel_type: "slack",
        config: {},
        enabled: true,
        sort_index: 0,
      });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalledWith("Create failed");
  });
});

describe("useUpdateChannel", () => {
  it("updates a channel and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useUpdateChannel(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "ch-1", channel: { name: "Updated" } });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["channels"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/update_channel`, () =>
        new HttpResponse("Update failed", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useUpdateChannel(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "ch-1", channel: { name: "Fail" } });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalledWith("Update failed");
  });
});

describe("useDeleteChannel", () => {
  it("deletes a channel and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteChannel(), { wrapper });

    await act(async () => {
      result.current.mutate("ch-1");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["channels"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_channel`, () =>
        new HttpResponse("Delete failed", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteChannel(), { wrapper });

    await act(async () => {
      result.current.mutate("ch-1");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalledWith("Delete failed");
  });
});

describe("useTestChannel", () => {
  it("tests a channel successfully", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useTestChannel(), { wrapper });

    await act(async () => {
      result.current.mutate("ch-1");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.success).toBe(true);
  });
});

// ── Event Types ──

describe("useCreateEventType", () => {
  it("creates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCreateEventType(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "New Event",
        category: "custom",
        is_builtin: false,
        config: {},
        enabled: true,
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["eventTypes"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/create_event_type`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateEventType(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "Fail",
        category: "custom",
        is_builtin: false,
        config: {},
        enabled: true,
      });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useUpdateEventType", () => {
  it("updates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useUpdateEventType(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "evt-1", eventType: { enabled: false } });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["eventTypes"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/update_event_type`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useUpdateEventType(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "evt-1", eventType: { enabled: false } });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useDeleteEventType", () => {
  it("deletes and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteEventType(), { wrapper });

    await act(async () => {
      result.current.mutate("evt-4");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["eventTypes"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_event_type`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteEventType(), { wrapper });

    await act(async () => {
      result.current.mutate("evt-4");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── Rules ──

describe("useCreateRule", () => {
  it("creates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCreateRule(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "New Rule",
        rule_type: "regex",
        pattern: ".*",
        event_type_id: "evt-1",
        enabled: true,
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["rules"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/create_rule`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateRule(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "Fail",
        rule_type: "keyword",
        pattern: "x",
        event_type_id: "evt-1",
        enabled: true,
      });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useUpdateRule", () => {
  it("updates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useUpdateRule(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "rule-1", rule: { enabled: false } });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["rules"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/update_rule`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useUpdateRule(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "rule-1", rule: { enabled: false } });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useDeleteRule", () => {
  it("deletes and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteRule(), { wrapper });

    await act(async () => {
      result.current.mutate("rule-1");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["rules"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_rule`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteRule(), { wrapper });

    await act(async () => {
      result.current.mutate("rule-1");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── Routing ──

describe("useSetRouting", () => {
  it("sets routing and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useSetRouting(), { wrapper });

    await act(async () => {
      result.current.mutate({
        event_type_id: "evt-3",
        channel_id: "ch-1",
        enabled: true,
        priority: 0,
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["routing"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/set_routing`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useSetRouting(), { wrapper });

    await act(async () => {
      result.current.mutate({
        event_type_id: "evt-3",
        channel_id: "ch-1",
        enabled: true,
        priority: 0,
      });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useDeleteRouting", () => {
  it("deletes routing and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteRouting(), { wrapper });

    await act(async () => {
      result.current.mutate({ eventTypeId: "evt-1", channelId: "ch-1" });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["routing"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_routing`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteRouting(), { wrapper });

    await act(async () => {
      result.current.mutate({ eventTypeId: "evt-1", channelId: "ch-1" });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── Templates ──

describe("useCreateTemplate", () => {
  it("creates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useCreateTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "New Tpl",
        channel_type: "discord",
        body_template: "{{msg}}",
        format: "text",
        is_default: false,
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["templates"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/create_template`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate({
        name: "Fail",
        channel_type: "slack",
        body_template: "",
        format: "text",
        is_default: false,
      });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useUpdateTemplate", () => {
  it("updates and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useUpdateTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate({
        id: "tpl-1",
        template: { name: "Updated" },
      });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["templates"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/update_template`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useUpdateTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate({ id: "tpl-1", template: { name: "Fail" } });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useDeleteTemplate", () => {
  it("deletes and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate("tpl-1");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["templates"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_template`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteTemplate(), { wrapper });

    await act(async () => {
      result.current.mutate("tpl-1");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── History ──

describe("useClearHistory", () => {
  it("clears and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useClearHistory(), { wrapper });

    await act(async () => {
      result.current.mutate();
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["history"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/clear_history`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useClearHistory(), { wrapper });

    await act(async () => {
      result.current.mutate();
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── Settings ──

describe("useSetSetting", () => {
  it("sets and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useSetSetting(), { wrapper });

    await act(async () => {
      result.current.mutate({ key: "language", value: "zh" });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["settings"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/set_setting`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useSetSetting(), { wrapper });

    await act(async () => {
      result.current.mutate({ key: "language", value: "zh" });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useDeleteSetting", () => {
  it("deletes and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useDeleteSetting(), { wrapper });

    await act(async () => {
      result.current.mutate("voice_name");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["settings"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/delete_setting`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteSetting(), { wrapper });

    await act(async () => {
      result.current.mutate("voice_name");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

// ── Hooks ──

describe("useInstallHook", () => {
  it("installs and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useInstallHook(), { wrapper });

    await act(async () => {
      result.current.mutate("codex");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["hooksStatus"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/install_hook`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useInstallHook(), { wrapper });

    await act(async () => {
      result.current.mutate("codex");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});

describe("useUninstallHook", () => {
  it("uninstalls and invalidates cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    const spy = vi.spyOn(queryClient, "invalidateQueries");
    const { result } = renderHook(() => useUninstallHook(), { wrapper });

    await act(async () => {
      result.current.mutate("claude");
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(spy).toHaveBeenCalledWith({ queryKey: ["hooksStatus"] });
  });

  it("calls toast.error on failure", async () => {
    server.use(
      http.post(`${TAURI_ENDPOINT}/uninstall_hook`, () =>
        new HttpResponse("Fail", { status: 500 }),
      ),
    );
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useUninstallHook(), { wrapper });

    await act(async () => {
      result.current.mutate("claude");
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
  });
});
