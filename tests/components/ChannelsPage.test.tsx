import { describe, it, expect, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ChannelsPage } from "@/components/channels/ChannelsPage";
import { renderWithProviders } from "../utils/renderWithProviders";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

describe("ChannelsPage", () => {
  // ── Rendering ──

  it("renders title and add button", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.title")).toBeInTheDocument();
    });
    expect(screen.getByText("channels.add")).toBeInTheDocument();
  });

  it("renders channel names", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });
    expect(screen.getByText("Slack Alerts")).toBeInTheDocument();
    expect(screen.getByText("Discord Logs")).toBeInTheDocument();
  });

  it("renders channel type badges", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      // t("channels.types.native", "native") returns "native" as default value
      expect(screen.getByText("native")).toBeInTheDocument();
    });
    expect(screen.getByText("slack")).toBeInTheDocument();
    expect(screen.getByText("discord")).toBeInTheDocument();
  });

  it("renders enable/disable switches", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      const switches = screen.getAllByRole("switch");
      expect(switches).toHaveLength(4);
    });
  });

  it("shows enabled/disabled labels", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("channels.enabled")).toHaveLength(2);
    });
    expect(screen.getByText("channels.disabled")).toBeInTheDocument();
  });

  it("renders test, edit, and delete buttons for each channel", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("channels.test")).toHaveLength(3);
    });
    expect(screen.getAllByText("common.edit")).toHaveLength(3);
    expect(screen.getAllByText("common.delete")).toHaveLength(3);
  });

  it("disables test button for disabled channels", async () => {
    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      const testButtons = screen.getAllByText("channels.test");
      expect(testButtons).toHaveLength(3);
    });
    // Discord (ch-3) is disabled, so its test button should be disabled
    const testButtons = screen.getAllByText("channels.test").map(
      (el) => el.closest("button")!,
    );
    const disabledButtons = testButtons.filter((btn) => btn.disabled);
    expect(disabledButtons).toHaveLength(1);
  });

  it("shows empty state when no channels", async () => {
    const { channelsApi } = await import("@/lib/api/channels");
    await channelsApi.delete("ch-1");
    await channelsApi.delete("ch-2");
    await channelsApi.delete("ch-3");

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.empty")).toBeInTheDocument();
    });
  });

  it("does not show feishu as a standalone channel type", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.add")).toBeInTheDocument();
    });

    const addButtons = screen.getAllByText("channels.add");
    await user.click(addButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("channels.type")).toBeInTheDocument();
    });

    // feishu should not appear in the type selector options
    expect(screen.queryByText("channels.types.feishu")).not.toBeInTheDocument();
  });

  // ── Toggle ──

  it("toggles channel enabled status", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });

    const switches = screen.getAllByRole("switch");
    // switches[0] is global kill switch, use first channel toggle.
    await user.click(switches[1]);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  // ── Test ──

  it("tests a channel", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("channels.test")).toHaveLength(3);
    });

    // Click first enabled test button (Native Notification)
    const testButtons = screen.getAllByText("channels.test").map(
      (el) => el.closest("button")!,
    );
    const enabledTestButton = testButtons.find((btn) => !btn.disabled);
    await user.click(enabledTestButton!);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  // ── Delete ──

  it("deletes a channel on confirm", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByText("common.delete");
    await user.click(deleteButtons[0]);

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("channels.deleteConfirm")).toBeInTheDocument();
    });

    // Click confirm button (use role to disambiguate from title)
    const confirmButton = screen.getByRole("button", { name: "common.confirm" });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(
        screen.queryByText("Native Notification"),
      ).not.toBeInTheDocument();
    });
  });

  it("does not delete when confirm cancelled", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByText("common.delete");
    await user.click(deleteButtons[0]);

    // Wait for confirm dialog to appear
    await waitFor(() => {
      expect(screen.getByText("channels.deleteConfirm")).toBeInTheDocument();
    });

    // Click cancel button
    await user.click(screen.getByText("common.cancel"));

    await waitFor(() => {
      expect(screen.queryByText("channels.deleteConfirm")).not.toBeInTheDocument();
    });
    expect(screen.getByText("Native Notification")).toBeInTheDocument();
  });

  // ── Create Dialog ──

  it("opens create dialog with empty form", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.add")).toBeInTheDocument();
    });

    // Click the add button (the first one, in the header)
    const addButtons = screen.getAllByText("channels.add");
    await user.click(addButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("channels.name")).toBeInTheDocument();
    });
    expect(screen.getByText("channels.type")).toBeInTheDocument();
    expect(screen.getByText("common.save")).toBeInTheDocument();
    expect(screen.getByText("common.cancel")).toBeInTheDocument();
  });

  it("closes dialog on cancel", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.add")).toBeInTheDocument();
    });

    const addButtons = screen.getAllByText("channels.add");
    await user.click(addButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("common.cancel")).toBeInTheDocument();
    });

    await user.click(screen.getByText("common.cancel"));

    await waitFor(() => {
      expect(screen.queryByText("channels.name")).not.toBeInTheDocument();
    });
  });

  it("save button is disabled when form is empty", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("channels.add")).toBeInTheDocument();
    });

    const addButtons = screen.getAllByText("channels.add");
    await user.click(addButtons[0]);

    await waitFor(() => {
      const saveButton = screen.getByText("common.save").closest("button")!;
      expect(saveButton).toBeDisabled();
    });
  });

  // ── Edit Dialog ──

  it("opens edit dialog with pre-filled data", async () => {
    const user = userEvent.setup();

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });

    const editButtons = screen.getAllByText("common.edit");
    await user.click(editButtons[0]);

    await waitFor(() => {
      // The edit dialog title should show "common.edit"
      const nameInput = screen.getByPlaceholderText("channels.name");
      expect(nameInput).toHaveValue("Native Notification");
    });
  });

  it("saves updated channel", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<ChannelsPage />);
    await waitFor(() => {
      expect(screen.getByText("Native Notification")).toBeInTheDocument();
    });

    const editButtons = screen.getAllByText("common.edit");
    await user.click(editButtons[0]);

    await waitFor(() => {
      expect(screen.getByPlaceholderText("channels.name")).toBeInTheDocument();
    });

    const nameInput = screen.getByPlaceholderText("channels.name");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Native");

    await user.click(screen.getByText("common.save"));

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });
});
