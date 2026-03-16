import { describe, it, expect, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RulesPage } from "@/components/rules/RulesPage";
import { renderWithProviders } from "../utils/renderWithProviders";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

describe("RulesPage", () => {
  it("renders title and add button", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("rules.title")).toBeInTheDocument();
    });
    expect(screen.getByText("rules.add")).toBeInTheDocument();
  });

  it("renders rule list with names", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("Error Keyword")).toBeInTheDocument();
    });
    expect(screen.getByText("Source File Change")).toBeInTheDocument();
  });

  it("renders rule type badges", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      // t("rules.types.keyword", "keyword") returns "keyword"
      expect(screen.getByText("keyword")).toBeInTheDocument();
    });
    expect(screen.getByText("file_change")).toBeInTheDocument();
  });

  it("renders pattern for each rule", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("error")).toBeInTheDocument();
    });
    expect(screen.getByText("src/**/*.ts")).toBeInTheDocument();
  });

  it("renders linked event type names", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      // Event type names are rendered inline with label: "rules.eventType: Error Detected"
      expect(screen.getByText(/Error Detected/)).toBeInTheDocument();
    });
    expect(screen.getByText(/Long Running Task/)).toBeInTheDocument();
  });

  it("renders enable/disable switches", async () => {
    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      const switches = screen.getAllByRole("switch");
      expect(switches.length).toBeGreaterThanOrEqual(2);
    });
  });

  it("toggles rule enabled status", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("Error Keyword")).toBeInTheDocument();
    });

    const switches = screen.getAllByRole("switch");
    await user.click(switches[0]);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  it("deletes a rule on confirm", async () => {
    const user = userEvent.setup();

    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("Error Keyword")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByText("common.delete");
    await user.click(deleteButtons[0]);

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("rules.deleteConfirm")).toBeInTheDocument();
    });

    // Click confirm button (use role to disambiguate from title)
    const confirmButton = screen.getByRole("button", { name: "common.confirm" });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(screen.queryByText("Error Keyword")).not.toBeInTheDocument();
    });
  });

  it("does not delete when confirm cancelled", async () => {
    const user = userEvent.setup();

    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("Error Keyword")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByText("common.delete");
    await user.click(deleteButtons[0]);

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("rules.deleteConfirm")).toBeInTheDocument();
    });

    // Click cancel button
    await user.click(screen.getByText("common.cancel"));

    await waitFor(() => {
      expect(screen.queryByText("rules.deleteConfirm")).not.toBeInTheDocument();
    });
    // Rule should still exist
    expect(screen.getByText("Error Keyword")).toBeInTheDocument();
  });

  it("opens create dialog", async () => {
    const user = userEvent.setup();

    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("rules.add")).toBeInTheDocument();
    });

    await user.click(screen.getByText("rules.add"));

    await waitFor(() => {
      expect(screen.getByText("rules.name")).toBeInTheDocument();
    });
    expect(screen.getByText("rules.type")).toBeInTheDocument();
    expect(screen.getByText("rules.pattern")).toBeInTheDocument();
    expect(screen.getByText("rules.eventType")).toBeInTheDocument();
  });

  it("shows empty state when no rules", async () => {
    const { rulesApi } = await import("@/lib/api/rules");
    await rulesApi.delete("rule-1");
    await rulesApi.delete("rule-2");

    renderWithProviders(<RulesPage />);
    await waitFor(() => {
      expect(screen.getByText("rules.empty")).toBeInTheDocument();
    });
  });
});
