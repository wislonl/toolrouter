import { invoke } from "@tauri-apps/api/core";

export type VscodePluginTargetId = "claude" | "codex" | "kilo" | "opencode";

export type VscodePluginStatusKind =
  | "not_installed"
  | "detected"
  | "managed"
  | "error";

export interface VscodePluginStatus {
  id: VscodePluginTargetId;
  label: string;
  extensionId: string;
  installed: boolean;
  version?: string | null;
  configPaths: string[];
  status: VscodePluginStatusKind;
  message?: string | null;
}

export interface VscodePluginChangePreview {
  targetId: VscodePluginTargetId;
  paths: string[];
  summary: string;
  destructive: boolean;
}

export const vscodePluginsApi = {
  async listTargets(): Promise<VscodePluginStatus[]> {
    return await invoke("list_vscode_plugin_targets");
  },

  async readStatus(targetId: VscodePluginTargetId): Promise<VscodePluginStatus> {
    return await invoke("read_vscode_plugin_status", { targetId });
  },

  async syncProvider(targetId: VscodePluginTargetId): Promise<boolean> {
    return await invoke("sync_vscode_plugin_provider", { targetId });
  },

  async clearProvider(targetId: VscodePluginTargetId): Promise<boolean> {
    return await invoke("clear_vscode_plugin_provider", { targetId });
  },

  async previewChanges(
    targetId: VscodePluginTargetId,
  ): Promise<VscodePluginChangePreview> {
    return await invoke("preview_vscode_plugin_changes", { targetId });
  },
};
