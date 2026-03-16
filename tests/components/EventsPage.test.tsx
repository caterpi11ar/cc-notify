import { describe, it, expect, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EventsPage } from "@/components/events/EventsPage";
import { renderWithProviders } from "../utils/renderWithProviders";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

describe("EventsPage", () => {
  it("renders title", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      expect(screen.getByText("events.title")).toBeInTheDocument();
    });
  });

  it("renders events grouped by category", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      // t("events.categories.claude_hook", "claude_hook") returns "claude_hook"
      // Appears as heading + per-event badges
      expect(screen.getAllByText("claude_hook").length).toBeGreaterThanOrEqual(1);
    });
    expect(screen.getAllByText("extended").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("custom").length).toBeGreaterThanOrEqual(1);
  });

  it("renders event names", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      expect(screen.getByText("Task Complete")).toBeInTheDocument();
    });
    expect(screen.getByText("Error Detected")).toBeInTheDocument();
    expect(screen.getByText("Long Running Task")).toBeInTheDocument();
    expect(screen.getByText("Custom Alert")).toBeInTheDocument();
  });

  it("renders built-in badge for built-in events", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("Built-in")).toHaveLength(2);
    });
  });

  it("renders enable/disable switches for events", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      const switches = screen.getAllByRole("switch");
      // 4 events = 4 switches
      expect(switches.length).toBe(4);
    });
  });

  it("renders routing checkboxes for channels", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      // 3 channels × 4 events = 12 checkboxes
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes).toHaveLength(12);
    });
  });

  it("renders channel names in routing section", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      // Each event card shows all 3 channel names
      const nativeTexts = screen.getAllByText("Native Notification");
      expect(nativeTexts.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("routing checkboxes reflect routing state", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes.length).toBe(12);
    });
    // evt-1 is routed to ch-1 and ch-2, so first event's first two checkboxes should be checked
    // This verifies the routing map is applied correctly
    const checkboxes = screen.getAllByRole("checkbox");
    // At least some should be checked based on our routing fixtures
    const checkedCount = checkboxes.filter(
      (cb) => cb.getAttribute("data-state") === "checked",
    ).length;
    // We have 3 routings enabled: evt-1→ch-1, evt-1→ch-2, evt-2→ch-2
    expect(checkedCount).toBe(3);
  });

  it("toggles event enabled status", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      expect(screen.getByText("Task Complete")).toBeInTheDocument();
    });

    const switches = screen.getAllByRole("switch");
    await user.click(switches[0]);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  it("toggles routing checkbox to add routing", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes.length).toBe(12);
    });

    // Find an unchecked checkbox and click it
    const checkboxes = screen.getAllByRole("checkbox");
    const unchecked = checkboxes.find(
      (cb) => cb.getAttribute("data-state") === "unchecked",
    );
    expect(unchecked).toBeTruthy();
    await user.click(unchecked!);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  it("toggles routing checkbox to remove routing", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes.length).toBe(12);
    });

    // Find a checked checkbox and click it
    const checkboxes = screen.getAllByRole("checkbox");
    const checked = checkboxes.find(
      (cb) => cb.getAttribute("data-state") === "checked",
    );
    expect(checked).toBeTruthy();
    await user.click(checked!);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  it("shows empty state when no events", async () => {
    const { eventTypesApi } = await import("@/lib/api/eventTypes");
    await eventTypesApi.delete("evt-1");
    await eventTypesApi.delete("evt-2");
    await eventTypesApi.delete("evt-3");
    await eventTypesApi.delete("evt-4");

    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      expect(screen.getByText("events.empty")).toBeInTheDocument();
    });
  });

  it("disables routing checkboxes for disabled events", async () => {
    renderWithProviders(<EventsPage />);
    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      expect(checkboxes.length).toBe(12);
    });
    // evt-4 (Custom Alert) is disabled, so its 3 checkboxes should be disabled
    const checkboxes = screen.getAllByRole("checkbox");
    const disabledCount = checkboxes.filter((cb) => cb.hasAttribute("disabled") || cb.getAttribute("data-disabled") === "").length;
    // 3 channels for 1 disabled event
    expect(disabledCount).toBe(3);
  });
});
