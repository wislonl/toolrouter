# ak-switch Design

Date: 2026-05-31

## Goal

Build `ak-switch` as a 1:1 copy of CC Switch with additional first-class support for:

- Claude Code
- Codex
- VS Code as an editor/plugin host
- OpenClaw
- Hermes
- VS Code plugins for Claude, Codex, Kilo, and OpenCode

The baseline implementation should be copied from the upstream MIT project:

- Repository: `farion1231/cc-switch`
- Baseline tag: `v3.16.0`
- Baseline commit: `47232cb`

The copied project must retain the MIT license and attribution.

## Current Evidence

The upstream `v3.16.0` application is a Tauri 2 + React app. It already registers these managed app IDs:

- `claude`
- `claude-desktop`
- `codex`
- `gemini`
- `opencode`
- `openclaw`
- `hermes`

OpenClaw and Hermes are therefore not greenfield additions. They must be preserved and verified after the copy.

The upstream VS Code integration is currently narrow:

- It exposes a setting named `enableClaudePluginIntegration`.
- It writes `~/.claude/config.json`.
- The managed field is `primaryApiKey: "any"` for non-official Claude provider routing.

The local VS Code installation currently includes:

- `Anthropic.claude-code` (`Claude Code for VS Code`)
- `openai.chatgpt` (`Codex - OpenAI's coding agent`)

No Kilo or OpenCode VS Code extension was found locally during this pass.

## Product Shape

`ak-switch` should keep the upstream layout and behavior as much as possible:

- Same provider model.
- Same switching workflow.
- Same tray/window behavior.
- Same local proxy architecture.
- Same config backup posture.
- Same OpenClaw and Hermes panels.

The primary visible addition should be a new settings/management surface named `VS Code Plugins`.

This surface should detect, display, and optionally synchronize supported VS Code plugins:

- Claude Code extension
- Codex extension
- Kilo Code extension
- OpenCode extension or wrapper

Each plugin should be handled by a small backend adapter. The UI should not write plugin files directly.

## VS Code Plugin Adapters

### Claude Plugin

Supported targets:

- VS Code User Settings:
  - macOS: `~/Library/Application Support/Code/User/settings.json`
  - Linux: `~/.config/Code/User/settings.json`
  - Windows: `%APPDATA%/Code/User/settings.json`
- Claude shared config:
  - `~/.claude/config.json`
  - `~/.claude/settings.json`

Fields:

- `claudeCode.environmentVariables`
- `claudeCode.disableLoginPrompt`
- Existing upstream `primaryApiKey: "any"` behavior in `~/.claude/config.json`

Sync behavior:

- For a third-party Claude provider, write environment variables such as base URL and auth token into the VS Code setting array.
- For official Claude login, remove only fields previously managed by `ak-switch`.
- Preserve unrelated user settings.
- Never log or document actual token values.

### Codex Plugin

Supported target:

- Official OpenAI VS Code extension ID: `openai.chatgpt`

Observed settings are mostly UI/runtime preferences such as `chatgpt.cliExecutable` and do not expose provider API fields in VS Code `package.json`.

Sync behavior:

- Treat the extension as a host for the Codex CLI/runtime.
- Reuse existing Codex config support:
  - `~/.codex/config.toml`
  - `~/.codex/auth.json`
  - model catalog files used by upstream CC Switch
- Optionally detect the extension install and show whether the configured Codex CLI path is default or overridden.

### Kilo Plugin

Supported target:

- User config: `~/.config/kilo/kilo.jsonc`
- Project config candidates:
  - `kilo.jsonc`
  - `.kilo/kilo.jsonc`

Sync behavior:

- Manage only `provider.ak-switch-current` in Kilo's JSONC config.
- Source the payload from the current AK Switch OpenCode provider and mark it with `metadata.managedBy: "ak-switch"`.
- Create a timestamped backup before writing normalized JSONC.
- Keep Kilo support optional when the extension is not installed locally.

### OpenCode Plugin

Supported targets:

- Official VS Code extension ID:
  - `sst-dev.opencode`
  - keep legacy detection compatibility with `opencode.opencode-*` folders
- User config: `~/.config/opencode/opencode.json`
- User auth: `~/.local/share/opencode/auth.json`
- Project config: `opencode.json`

Sync behavior:

- Reuse upstream OpenCode support as the source of truth.
- Add VS Code plugin detection and status when the extension/wrapper is present.
- Manage only `provider.ak-switch-current` in `opencode.json` and mark it with `metadata.managedBy: "ak-switch"`.
- Do not duplicate credentials into VS Code settings unless a specific extension requires it.

## Backend Design

The first implementation keeps the adapter code in `src-tauri/src/commands/vscode_plugins.rs` so it matches the existing command-module layout. Split it into smaller modules only if the adapter surface grows.

Add Tauri commands:

- `list_vscode_plugin_targets`
- `read_vscode_plugin_status`
- `sync_vscode_plugin_provider`
- `clear_vscode_plugin_provider`
- `preview_vscode_plugin_changes`

Every write command must:

- Make a timestamped backup.
- Preserve unrelated fields.
- Mark managed fields so they can be removed later.
- Avoid printing secrets to logs.

## Frontend Design

Add a `VS Code Plugins` section in Settings or Extensions.

The panel should show:

- Plugin name.
- Detected extension ID and version.
- Config path.
- Current status: installed, not installed, configured, managed, unmanaged, error.
- Last sync result.
- Actions: Sync current provider, Clear managed config, Open config file.

Do not create a separate app ID named `vscode` until a real provider model is needed. For now, VS Code is an integration host with plugin adapters. This avoids forcing VS Code into the same model as Claude/Codex/OpenClaw/Hermes when it does not own a single provider config format.

## Branding Copy

Initial rename scope:

- `cc-switch` package name to `ak-switch`
- Product name `CC Switch` to `AK Switch`
- Identifier `com.ccswitch.desktop` to `com.akswitch.desktop`
- Deep link scheme `ccswitch` to `akswitch`

Keep original license and add an attribution section:

> AK Switch is based on CC Switch by Jason Young, licensed under the MIT License.

## Testing Plan

Backend unit tests:

- Claude adapter preserves unrelated VS Code settings.
- Claude adapter removes only managed environment variables.
- Codex adapter reports the installed extension and Codex config paths.
- Kilo adapter can read/write/remove only AK Switch managed JSONC provider entries.
- OpenCode adapter can read/write/remove only AK Switch managed provider entries and preserve unrelated config.

Frontend unit tests:

- VS Code plugin panel renders all target states.
- Sync and clear actions call the correct Tauri commands.
- Missing plugins show actionable status without failing the whole panel.

Integration checks:

- `pnpm run typecheck`
- `pnpm run test:unit`
- `pnpm run build:renderer`
- Tauri Rust tests for new adapters.

Manual verification still required:

- Switch Claude provider and confirm VS Code Claude extension receives managed env vars.
- Switch Codex provider and confirm Codex extension still follows Codex CLI config.
- Install or fixture real Kilo/OpenCode VS Code extensions and confirm their plugins consume the managed shared config entry.
- Confirm OpenClaw and Hermes panels still work after branding/copy.

## Implementation Order

1. Copy upstream `v3.16.0` into this repository.
2. Preserve `LICENSE` and add attribution.
3. Rename package/product/identifier/deep-link/update metadata to `ak-switch`.
4. Run baseline install/typecheck/tests.
5. Add backend VS Code plugin adapter tests first.
6. Implement backend adapters.
7. Add frontend tests for the new panel.
8. Implement the panel and wire commands.
9. Verify build and unit tests.
10. Leave a handoff note with current status, known gaps, and manual validation steps.

## Open Risks

- Kilo and OpenCode VS Code extension IDs were not present locally, so detection must rely on known IDs plus a configurable fallback.
- VS Code extensions may change private storage or CLI launching behavior. The first implementation should write only documented/user-facing config files.
- Secrets must never be copied into docs, tests, logs, or memory.
- Upstream updater metadata must not keep pointing to CC Switch releases after rebranding.
- Kilo/OpenCode currently source from the AK Switch OpenCode provider. A later provider-source picker may be needed if users expect Kilo to sync from non-OpenCode app providers.
