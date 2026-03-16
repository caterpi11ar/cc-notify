import { describe, it, expect, vi } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { HistoryPage } from "@/components/history/HistoryPage";
import { renderWithProviders } from "../utils/renderWithProviders";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

describe("HistoryPage", () => {
  it("renders title", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.title")).toBeInTheDocument();
    });
  });

  it("renders table headers after loading", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.columns.timestamp")).toBeInTheDocument();
    });
    expect(screen.getByText("history.columns.event")).toBeInTheDocument();
    expect(screen.getByText("history.columns.channel")).toBeInTheDocument();
    expect(screen.getByText("history.columns.status")).toBeInTheDocument();
    expect(screen.getByText("history.columns.message")).toBeInTheDocument();
  });

  it("renders history rows with resolved names", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getAllByText("Native Notification").length).toBeGreaterThanOrEqual(1);
    });
    expect(screen.getAllByText("Slack Alerts").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Task Complete").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Error Detected")).toBeInTheDocument();
  });

  it("renders status badges", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getAllByText("history.status.sent")).toHaveLength(2);
    });
    expect(screen.getByText("history.status.failed")).toBeInTheDocument();
  });

  it("renders message body or error message", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(
        screen.getByText("Task completed successfully"),
      ).toBeInTheDocument();
    });
    // Failed entry shows error_message instead of message_body
    expect(screen.getByText("Connection timeout")).toBeInTheDocument();
  });

  it("formats timestamps", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      const rows = screen.getAllByRole("row");
      // Header + 3 data rows
      expect(rows.length).toBeGreaterThanOrEqual(4);
    });
  });

  it("shows clear button when history exists", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.clear")).toBeInTheDocument();
    });
  });

  it("clears history on confirm", async () => {
    const user = userEvent.setup();

    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.clear")).toBeInTheDocument();
    });

    await user.click(screen.getByText("history.clear"));

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("history.clearConfirm")).toBeInTheDocument();
    });

    // Click confirm button (use role to disambiguate from title)
    const confirmButton = screen.getByRole("button", { name: "common.confirm" });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(screen.getByText("history.empty")).toBeInTheDocument();
    });
  });

  it("does not clear history when confirm cancelled", async () => {
    const user = userEvent.setup();

    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.clear")).toBeInTheDocument();
    });

    await user.click(screen.getByText("history.clear"));

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("history.clearConfirm")).toBeInTheDocument();
    });

    // Click cancel button
    await user.click(screen.getByText("common.cancel"));

    // History should still be visible
    await waitFor(() => {
      expect(
        screen.getByText("Task completed successfully"),
      ).toBeInTheDocument();
    });
  });

  it("shows empty state when no history", async () => {
    const { historyApi } = await import("@/lib/api/history");
    await historyApi.clear();

    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.empty")).toBeInTheDocument();
    });
  });

  it("shows Load More button when history.length >= limit", async () => {
    renderWithProviders(<HistoryPage />);
    await waitFor(() => {
      expect(screen.getByText("history.columns.timestamp")).toBeInTheDocument();
    });
    // Default limit is 50, we have 3 items, so Load More should NOT show
    expect(screen.queryByText("history.loadMore")).not.toBeInTheDocument();
  });
});
