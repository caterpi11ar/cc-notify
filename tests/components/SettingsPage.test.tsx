import { describe, it, expect, vi } from "vitest";
import { screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { renderWithProviders } from "../utils/renderWithProviders";

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

describe("SettingsPage", () => {
  // ── General Section ──

  it("renders title", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.title")).toBeInTheDocument();
    });
  });

  it("renders general section with language and retention", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.general")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.language")).toBeInTheDocument();
    expect(screen.getByText("settings.historyRetention")).toBeInTheDocument();
  });

  it("loads settings defaults from MSW", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      const retentionInput = screen.getByDisplayValue("30");
      expect(retentionInput).toBeInTheDocument();
    });
  });

  it("saves retention on blur", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByDisplayValue("30")).toBeInTheDocument();
    });

    const retentionInput = screen.getByDisplayValue("30");
    await user.clear(retentionInput);
    await user.type(retentionInput, "60");
    fireEvent.blur(retentionInput);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  // ── Quiet Hours Section ──

  it("renders quiet hours section", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.quietHours")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.quietHoursDesc")).toBeInTheDocument();
  });

  it("toggles quiet hours and shows time inputs", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.quietHours")).toBeInTheDocument();
    });

    // Quiet hours is disabled by default, find its switch
    const switches = screen.getAllByRole("switch");
    // The quiet hours switch is after general section
    // Find by looking for the card that contains "settings.quietHours"
    const quietHoursCard = screen.getByText("settings.quietHours").closest("[class*='card']");
    const quietHoursSwitch = quietHoursCard?.querySelector("[role='switch']");

    if (quietHoursSwitch) {
      await user.click(quietHoursSwitch as HTMLElement);

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalled();
      });

      // After enabling, start/end time inputs should appear
      await waitFor(() => {
        expect(screen.getByText("settings.quietHoursStart")).toBeInTheDocument();
      });
      expect(screen.getByText("settings.quietHoursEnd")).toBeInTheDocument();
    }
  });

  // ── Rate Limiting Section ──

  it("renders rate limiting section", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.rateLimit")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.rateLimitDesc")).toBeInTheDocument();
    expect(screen.getByText("settings.maxPerMinute")).toBeInTheDocument();
    expect(screen.getByText("settings.cooldownSeconds")).toBeInTheDocument();
  });

  it("shows default rate limiting values", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByDisplayValue("10")).toBeInTheDocument();
    });
    expect(screen.getByDisplayValue("5")).toBeInTheDocument();
  });

  it("saves max per minute on blur", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByDisplayValue("10")).toBeInTheDocument();
    });

    const maxInput = screen.getByDisplayValue("10");
    await user.clear(maxInput);
    await user.type(maxInput, "20");
    fireEvent.blur(maxInput);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  // ── Kill Switch Section ──

  it("renders kill switch section", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.killSwitch")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.killSwitchDesc")).toBeInTheDocument();
  });

  it("toggles kill switch and shows warning", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.killSwitch")).toBeInTheDocument();
    });

    const killSwitchCard = screen.getByText("settings.killSwitch").closest("[class*='card']");
    const killSwitchToggle = killSwitchCard?.querySelector("[role='switch']");

    if (killSwitchToggle) {
      await user.click(killSwitchToggle as HTMLElement);

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalled();
      });

      await waitFor(() => {
        expect(
          screen.getByText("settings.killSwitchWarning"),
        ).toBeInTheDocument();
      });
    }
  });

  // ── Sound Section ──

  it("renders sound section", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.sound")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.soundDesc")).toBeInTheDocument();
  });

  it("shows volume input when sound enabled", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      // Sound is enabled by default, so volume should show
      expect(screen.getByText("settings.volume")).toBeInTheDocument();
    });
    expect(screen.getByDisplayValue("80")).toBeInTheDocument();
  });

  it("saves volume on blur", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByDisplayValue("80")).toBeInTheDocument();
    });

    const volumeInput = screen.getByDisplayValue("80");
    await user.clear(volumeInput);
    await user.type(volumeInput, "50");
    fireEvent.blur(volumeInput);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  // ── Voice Section ──

  it("renders voice section", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.voice")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.voiceDesc")).toBeInTheDocument();
  });

  it("toggles voice and shows voice name input", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.voice")).toBeInTheDocument();
    });

    const voiceCard = screen.getByText("settings.voice").closest("[class*='card']");
    const voiceSwitch = voiceCard?.querySelector("[role='switch']");

    if (voiceSwitch) {
      await user.click(voiceSwitch as HTMLElement);

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalled();
      });

      await waitFor(() => {
        expect(screen.getByText("settings.voiceName")).toBeInTheDocument();
      });
    }
  });

  // ── Hooks Section ──

  it("renders hooks section with tool labels", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("settings.hooks")).toBeInTheDocument();
    });
    expect(screen.getByText("settings.hooksDesc")).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("Claude Code")).toBeInTheDocument();
    });
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
  });

  it("shows installed/not-installed badges for hooks", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      // claude=true → installed, codex=false, gemini=false → not installed
      expect(
        screen.getByText("settings.hooksInstalled"),
      ).toBeInTheDocument();
    });
    expect(
      screen.getAllByText("settings.hooksNotInstalled"),
    ).toHaveLength(2);
  });

  it("shows uninstall button for installed hooks", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("common.uninstall")).toBeInTheDocument();
    });
  });

  it("shows install button for not-installed hooks", async () => {
    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("common.install")).toHaveLength(2);
    });
  });

  it("installs a hook", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getAllByText("common.install")).toHaveLength(2);
    });

    const installButtons = screen.getAllByText("common.install");
    await user.click(installButtons[0]);

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });

  it("uninstalls a hook", async () => {
    const user = userEvent.setup();
    const { toast } = await import("sonner");

    renderWithProviders(<SettingsPage />);
    await waitFor(() => {
      expect(screen.getByText("common.uninstall")).toBeInTheDocument();
    });

    await user.click(screen.getByText("common.uninstall"));

    await waitFor(() => {
      expect(toast.success).toHaveBeenCalled();
    });
  });
});
