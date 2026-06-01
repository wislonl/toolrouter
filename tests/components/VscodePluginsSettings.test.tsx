import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import { VscodePluginsSettings } from "@/components/settings/VscodePluginsSettings";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();
const listTargetsMock = vi.fn();
const syncProviderMock = vi.fn();
const clearProviderMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/lib/api", () => ({
  vscodePluginsApi: {
    listTargets: (...args: unknown[]) => listTargetsMock(...args),
    syncProvider: (...args: unknown[]) => syncProviderMock(...args),
    clearProvider: (...args: unknown[]) => clearProviderMock(...args),
  },
}));

const renderPanel = () => {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={client}>
      <VscodePluginsSettings />
    </QueryClientProvider>,
  );
};

describe("VscodePluginsSettings", () => {
  beforeEach(() => {
    listTargetsMock.mockReset();
    syncProviderMock.mockReset();
    clearProviderMock.mockReset();
    toastSuccessMock.mockReset();
    toastErrorMock.mockReset();
    listTargetsMock.mockResolvedValue([
      {
        id: "claude",
        label: "Claude Code",
        extensionId: "Anthropic.claude-code",
        installed: true,
        version: "1.2.3",
        configPaths: ["/mock/.claude/config.json"],
        status: "detected",
        message: null,
      },
      {
        id: "codex",
        label: "Codex",
        extensionId: "openai.chatgpt",
        installed: true,
        version: null,
        configPaths: ["/mock/.codex/config.toml"],
        status: "detected",
        message: "Using shared Codex config",
      },
      {
        id: "kilo",
        label: "Kilo",
        extensionId: "kilocode.kilo-code",
        installed: false,
        version: null,
        configPaths: ["/mock/.config/kilo/kilo.jsonc"],
        status: "not_installed",
        message: null,
      },
    ]);
    syncProviderMock.mockResolvedValue(true);
    clearProviderMock.mockResolvedValue(true);
  });

  it("renders plugin targets with status and config paths", async () => {
    renderPanel();

    expect(
      screen.getByText("settings.advanced.vscodePlugins.loading"),
    ).toBeInTheDocument();

    expect(await screen.findByText("Claude Code")).toBeInTheDocument();
    expect(
      screen.getByText("Anthropic.claude-code · 1.2.3"),
    ).toBeInTheDocument();
    expect(screen.getByText("/mock/.claude/config.json")).toBeInTheDocument();
    expect(screen.getByText("Using shared Codex config")).toBeInTheDocument();
    expect(screen.getByText("Kilo")).toBeInTheDocument();
  });

  it("syncs and clears writable installed plugin targets", async () => {
    renderPanel();

    await screen.findByText("Claude Code");
    const syncButtons = screen.getAllByRole("button", {
      name: /settings\.advanced\.vscodePlugins\.sync/,
    });
    const clearButtons = screen.getAllByRole("button", {
      name: /settings\.advanced\.vscodePlugins\.clear/,
    });

    fireEvent.click(syncButtons[0]);
    await waitFor(() => expect(syncProviderMock).toHaveBeenCalledWith("claude"));
    expect(toastSuccessMock).toHaveBeenCalledWith(
      "settings.advanced.vscodePlugins.syncSuccess",
    );

    fireEvent.click(clearButtons[0]);
    await waitFor(() =>
      expect(clearProviderMock).toHaveBeenCalledWith("claude"),
    );
    expect(toastSuccessMock).toHaveBeenCalledWith(
      "settings.advanced.vscodePlugins.clearSuccess",
    );
  });

  it("disables sync and clear for Codex shared-config target", async () => {
    renderPanel();

    await screen.findByText("Codex");
    const syncButtons = screen.getAllByRole("button", {
      name: /settings\.advanced\.vscodePlugins\.sync/,
    });
    const clearButtons = screen.getAllByRole("button", {
      name: /settings\.advanced\.vscodePlugins\.clear/,
    });

    expect(clearButtons[1]).toBeDisabled();
    expect(syncButtons[1]).toBeDisabled();

    fireEvent.click(syncButtons[1]);
    fireEvent.click(clearButtons[1]);
    expect(syncProviderMock).not.toHaveBeenCalledWith("codex");
    expect(clearProviderMock).not.toHaveBeenCalledWith("codex");
  });

  it("disables actions for missing extensions", async () => {
    renderPanel();

    await screen.findByText("Kilo");
    const syncButtons = screen.getAllByRole("button", {
      name: /settings\.advanced\.vscodePlugins\.sync/,
    });

    expect(syncButtons[2]).toBeDisabled();
  });
});
