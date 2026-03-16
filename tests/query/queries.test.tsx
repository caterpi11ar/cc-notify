import { describe, it, expect } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { createTestQueryClient } from "../utils/testQueryClient";
import {
  useChannelsQuery,
  useEventTypesQuery,
  useRulesQuery,
  useRoutingQuery,
  useTemplatesQuery,
  useHistoryQuery,
  useSettingsQuery,
  useHooksStatusQuery,
} from "@/lib/query";

function createWrapper() {
  const queryClient = createTestQueryClient();
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  };
}

describe("useChannelsQuery", () => {
  it("returns channels from MSW", async () => {
    const { result } = renderHook(() => useChannelsQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(3);
    expect(result.current.data![0].name).toBe("Native Notification");
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useChannelsQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    const cached = queryClient.getQueryData(["channels"]);
    expect(cached).toHaveLength(3);
  });
});

describe("useEventTypesQuery", () => {
  it("returns event types from MSW", async () => {
    const { result } = renderHook(() => useEventTypesQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(4);
    expect(result.current.data![0].category).toBe("claude_hook");
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useEventTypesQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryClient.getQueryData(["eventTypes"])).toHaveLength(4);
  });
});

describe("useRulesQuery", () => {
  it("returns rules from MSW", async () => {
    const { result } = renderHook(() => useRulesQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(2);
    expect(result.current.data![0].rule_type).toBe("keyword");
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useRulesQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryClient.getQueryData(["rules"])).toHaveLength(2);
  });
});

describe("useRoutingQuery", () => {
  it("returns routings from MSW", async () => {
    const { result } = renderHook(() => useRoutingQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(3);
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useRoutingQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryClient.getQueryData(["routing"])).toHaveLength(3);
  });
});

describe("useTemplatesQuery", () => {
  it("returns templates from MSW", async () => {
    const { result } = renderHook(() => useTemplatesQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0].name).toBe("Default Slack");
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useTemplatesQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryClient.getQueryData(["templates"])).toHaveLength(1);
  });
});

describe("useHistoryQuery", () => {
  it("returns history with default params", async () => {
    const { result } = renderHook(() => useHistoryQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(3);
  });

  it("respects custom limit and offset", async () => {
    const { result } = renderHook(() => useHistoryQuery(2, 0), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(2);
  });

  it("uses correct queryKey with params", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useHistoryQuery(10, 1), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryClient.getQueryData(["history", 10, 1])).toHaveLength(2);
  });
});

describe("useSettingsQuery", () => {
  it("returns settings from MSW", async () => {
    const { result } = renderHook(() => useSettingsQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data!.language).toBe("en");
    expect(result.current.data!.kill_switch).toBe("false");
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useSettingsQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    const cached = queryClient.getQueryData(["settings"]) as Record<string, string>;
    expect(cached.language).toBe("en");
  });
});

describe("useHooksStatusQuery", () => {
  it("returns hooks status from MSW", async () => {
    const { result } = renderHook(() => useHooksStatusQuery(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data!.claude).toBe(true);
    expect(result.current.data!.codex).toBe(false);
    expect(result.current.data!.gemini).toBe(false);
  });

  it("uses correct queryKey", async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    const { result } = renderHook(() => useHooksStatusQuery(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    const cached = queryClient.getQueryData(["hooksStatus"]) as { claude: boolean };
    expect(cached.claude).toBe(true);
  });
});
