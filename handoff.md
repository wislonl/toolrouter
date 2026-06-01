# ak-switch Handoff

Date: 2026-05-31

## Goal

1:1 copy CC Switch and extend it for VS Code, OpenClaw, Hermes, and VS Code plugins for Claude, Codex, Kilo, and OpenCode.

## Current Status

- `/Users/jw/code/ak-switch` has been populated from upstream CC Switch.
- Upstream CC Switch was cloned only to `/tmp/ak-switch-upstream` for inspection.
- Upstream baseline identified as `farion1231/cc-switch` tag `v3.16.0`, commit `47232cb`.
- Upstream already supports OpenClaw and Hermes as managed app IDs.
- Upstream VS Code support was limited to Claude plugin integration via `~/.claude/config.json`.
- Local VS Code has `Anthropic.claude-code` and `openai.chatgpt` installed.
- Local VS Code did not show Kilo or OpenCode extensions in this pass.
- Runtime/application identity has been rebranded to AK Switch:
  - npm package: `ak-switch`
  - Rust package: `ak-switch`
  - Tauri product name: `AK Switch`
  - Tauri identifier: `com.akswitch.desktop`
  - deep link scheme: `akswitch`
  - app data path: `~/.ak-switch`
- Added a VS Code plugin settings panel under Settings -> Advanced:
  - detects Claude Code, Codex, Kilo, and OpenCode plugin targets
  - shows extension ID, version, status, and config paths
  - exposes sync/clear actions through `vscodePluginsApi`
- Added Tauri command registrations and an initial backend command module for VS Code plugin targets:
  - `list_vscode_plugin_targets`
  - `read_vscode_plugin_status`
  - `sync_vscode_plugin_provider`
  - `clear_vscode_plugin_provider`
  - `preview_vscode_plugin_changes`
- Current backend behavior:
  - Claude sync/clear reuses existing `~/.claude/config.json` managed-field logic and now also updates VS Code User `settings.json`:
    - writes `claudeCode.environmentVariables`
    - sets `claudeCode.disableLoginPrompt`
    - marks managed env entries with `managedBy: "ak-switch"` so clear only removes AK Switch owned values
    - creates timestamped backups before changing existing JSON files
  - Codex is detected as a shared-config target and currently requires no extra write because it reuses `~/.codex/config.toml` and `~/.codex/auth.json`; those files no longer make the VS Code plugin status show as AK-managed, and the UI disables Codex sync/clear no-op actions.
  - OpenCode sync/clear writes only an AK Switch managed entry at `provider.ak-switch-current` in `~/.config/opencode/opencode.json`.
  - Kilo sync/clear writes only an AK Switch managed entry at `provider.ak-switch-current` in `~/.config/kilo/kilo.jsonc`.
  - OpenCode/Kilo writes source the current AK Switch OpenCode provider and preserve unrelated config fields.
  - OpenCode/Kilo plugin status now reports `managed` only when `provider.ak-switch-current.metadata.managedBy` is `ak-switch`; user-owned config files no longer count as AK-managed.
- OpenCode VS Code detection now uses the Marketplace extension ID `sst-dev.opencode` and keeps `opencode.opencode-*` as legacy-compatible detection.
- VS Code plugin detection now parses extension versions before platform suffixes, so folders like `openai.chatgpt-26.5506.31421-darwin-arm64` report `26.5506.31421` instead of `arm64`.
- Claude VS Code plugin preview now describes both write surfaces it actually manages: `~/.claude/config.json` and VS Code User `settings.json` (`claudeCode.environmentVariables` / login prompt marker).
- Rust toolchain was installed through Homebrew `rustup` and the project toolchain `1.95`.
- JavaScript, renderer, and Rust verification pass.
- README first screens have been rebranded to AK Switch and `NOTICE.md` records upstream attribution.
- Release workflow artifact names and top-level README manual download names now use `AK-Switch-*`.
- Top-level English/Chinese README core sections now describe AK Switch, six managed tools including Hermes, VS Code plugin integration, `akswitch://`, and `~/.ak-switch`.
- Added a regression test so release workflow and README core branding/download/storage naming do not drift back to `CC-Switch-*`/`~/.cc-switch`.
- Added MSW fixtures for installed Skills and skill-storage migration so Settings integration tests no longer make unhandled `get_installed_skills` / `migrate_skill_storage` Tauri requests.
- Updated npm and Rust package descriptions to the expanded AK Switch scope: Claude Code, Codex, Gemini CLI, OpenCode, OpenClaw, Hermes Agent, and VS Code Plugins.
- Updated `SECURITY.md` and GitHub issue templates so public support/security entry points refer to AK Switch, not upstream `farion1231/cc-switch`, and added Hermes / VS Code Plugin options to app-scope dropdowns.
- Updated `.github/workflows/claude.yml` reviewer prompt so automated PR review context describes AK Switch and includes OpenCode, OpenClaw, Hermes, and VS Code plugin integrations.
- Added OpenClaw to the frontend Skills app surface (`SKILLS_APP_IDS`) now that the backend Skill service has OpenClaw skill directory support; MCP app IDs remain explicit and still exclude OpenClaw because backend MCP sync skips it.
- Updated the Rust panic hook stderr crash-log location message from `[CC-Switch]` to `[AK Switch]`; crash log path remains under `~/.ak-switch`.
- Made VS Code extension detection deterministic for OpenCode so the official `sst-dev.opencode-*` extension prefix is preferred over legacy/community `opencode.opencode-*` matches regardless of directory enumeration order.
- Updated top-level Star History and GitHub funding links to the AK Switch fork, and cleaned remaining production comments that used old CC Switch wording where no compatibility identifier was involved.

## Key Files

- `docs/ak-switch-design.md`: design, evidence, adapter boundaries, implementation order.
- `src/components/settings/VscodePluginsSettings.tsx`: VS Code plugin integration panel.
- `src/lib/api/vscodePlugins.ts`: frontend API wrapper and types for plugin commands.
- `src-tauri/src/commands/vscode_plugins.rs`: Tauri backend command module for detection, status, sync, clear, and previews.
- `tests/components/VscodePluginsSettings.test.tsx`: panel rendering and action coverage.
- `tests/integration/SettingsDialog.test.tsx`: Settings integration coverage, including Skill storage migration confirmation fed by installed Skill fixtures.
- `tests/config/akSwitchBranding.test.ts`: release workflow and top-level README branding/download/storage coverage.
- `tests/msw/handlers.ts` and `tests/msw/state.ts`: shared Tauri API fixtures for Settings/VS Code plugin tests.
- `NOTICE.md`: upstream CC Switch attribution and baseline.

## Decisions

- Use upstream MIT project as the copy baseline.
- Preserve MIT license and add attribution.
- Keep OpenClaw/Hermes support from upstream instead of reimplementing it.
- Add VS Code as an integration host with plugin adapters, not as a fake provider app ID.
- Manage plugin writes through Rust backend commands with backups and managed-field cleanup.
- Do not store or print secrets in docs, logs, tests, or handoff files.
- Treat Codex VS Code plugin as a consumer of existing `~/.codex` config/auth unless evidence shows a separate writable extension config.
- Treat OpenCode VS Code integrations as consumers of the existing OpenCode config/auth files unless a specific extension documents separate settings.
- Kilo uses JSONC config shared by the VS Code extension and CLI; write only an AK Switch managed provider entry and leave user/project entries untouched.
- Source Kilo/OpenCode plugin sync from the current AK Switch OpenCode provider until a separate source-selection UI is justified.
- Detect VS Code extensions by the declared prefix priority, not by filesystem entry order; this keeps official OpenCode detection stable while preserving legacy prefix compatibility.

## Verification

- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml`: passed, 1372 unit tests plus integration tests.
- `corepack pnpm run typecheck`: passed.
- `corepack pnpm run build:renderer`: passed.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts`: passed after extending README core-branding coverage, 1 file / 3 tests.
- `corepack pnpm run test:unit`: passed after extending the branding test, 53 files / 288 tests.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml commands::vscode_plugins::tests -- --nocapture`: passed after adding Codex shared-config status coverage, 7 tests.
- `corepack pnpm exec vitest run tests/components/VscodePluginsSettings.test.tsx`: passed after disabling Codex no-op sync/clear actions, 1 file / 4 tests.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml && corepack pnpm run typecheck`: passed after the VS Code plugin status change.
- `corepack pnpm run typecheck && corepack pnpm exec vitest run tests/components/VscodePluginsSettings.test.tsx tests/components/SettingsDialog.test.tsx`: passed after the Codex UI action change, 2 files / 12 tests.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml commands::vscode_plugins::tests -- --nocapture && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml && corepack pnpm exec vitest run tests/components/VscodePluginsSettings.test.tsx tests/integration/SettingsDialog.test.tsx`: passed after updating OpenCode to `sst-dev.opencode`, Rust 7 tests and frontend 2 files / 9 tests.
- `corepack pnpm exec vitest run tests/integration/SettingsDialog.test.tsx -t "uses installed skills"`: first failed with MSW warnings for missing `get_installed_skills` / `migrate_skill_storage`, then passed after adding handlers and Skill fixtures.
- `corepack pnpm exec vitest run tests/components/VscodePluginsSettings.test.tsx tests/integration/SettingsDialog.test.tsx && corepack pnpm run typecheck`: passed after the MSW Skill fixture update, frontend 2 files / 10 tests plus `tsc --noEmit`.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts -t "package metadata"`: first failed on the old package descriptions, then passed after updating `package.json` and `src-tauri/Cargo.toml`.
- `corepack pnpm exec prettier --check package.json tests/config/akSwitchBranding.test.ts tests/integration/SettingsDialog.test.tsx tests/msw/handlers.ts tests/msw/state.ts && corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts tests/components/VscodePluginsSettings.test.tsx tests/integration/SettingsDialog.test.tsx && corepack pnpm run typecheck && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after package metadata and MSW fixture updates, frontend 3 files / 14 tests plus `tsc --noEmit` and Cargo check.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts -t "security and issue"`: first failed on upstream CC Switch security/issue links, then passed after updating `SECURITY.md` and `.github/ISSUE_TEMPLATE/*`.
- `rg "farion1231/cc-switch|CC Switch Version|CC Switch 3\\.11\\.1|latest release of CC Switch" SECURITY.md .github/ISSUE_TEMPLATE -n`: no matches after the security/template update.
- `corepack pnpm exec prettier --check tests/config/akSwitchBranding.test.ts .github/ISSUE_TEMPLATE/bug_report.yml .github/ISSUE_TEMPLATE/config.yml .github/ISSUE_TEMPLATE/doc_issue.yml .github/ISSUE_TEMPLATE/feature_request.yml .github/ISSUE_TEMPLATE/question.yml`: passed.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts && corepack pnpm run typecheck`: passed after the security/template update, 1 file / 5 tests plus `tsc --noEmit`.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts -t "repository workflows"`: first failed on `.github/workflows/claude.yml` still describing `cc-switch`, then passed after updating the reviewer prompt.
- `corepack pnpm exec prettier --check tests/config/akSwitchBranding.test.ts .github/workflows/claude.yml .github/ISSUE_TEMPLATE/bug_report.yml .github/ISSUE_TEMPLATE/config.yml .github/ISSUE_TEMPLATE/doc_issue.yml .github/ISSUE_TEMPLATE/feature_request.yml .github/ISSUE_TEMPLATE/question.yml package.json && corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts && corepack pnpm run typecheck && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after the workflow/security/template updates, 1 file / 6 tests plus `tsc --noEmit` and Cargo check.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml commands::vscode_plugins::tests::claude_preview_mentions_vscode_settings_and_claude_config -- --nocapture`: first failed because Claude preview did not mention VS Code `settings.json`, then passed after updating the preview summary.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml commands::vscode_plugins::tests -- --nocapture && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after the Claude preview summary update, Rust plugin module 8 tests plus Cargo check.
- `corepack pnpm exec vitest run tests/config/appConfig.test.ts`: first failed because `SKILLS_APP_IDS` did not include `openclaw`, then passed after adding it and keeping `MCP_APP_IDS` explicit.
- `corepack pnpm exec prettier --check src/config/appConfig.tsx tests/config/appConfig.test.ts tests/components/UnifiedSkillsPanel.test.tsx && corepack pnpm run typecheck && corepack pnpm exec vitest run tests/config/appConfig.test.ts tests/components/UnifiedSkillsPanel.test.tsx tests/components/McpFormModal.test.tsx`: passed after OpenClaw Skills app-surface update, 3 files / 13 tests plus `tsc --noEmit`; `McpFormModal` still emits existing mock prop warnings.
- `corepack pnpm exec vitest run tests/config/appConfig.test.ts tests/components/UnifiedSkillsPanel.test.tsx tests/components/SkillsPageInstall.test.tsx tests/hooks/useImportSkillsFromApps.test.tsx && corepack pnpm run typecheck`: passed, 4 files / 10 tests plus `tsc --noEmit`.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml panic_hook::tests::crash_log_message_uses_ak_switch_brand -- --nocapture`: first failed because the crash-log stderr message still used `[CC-Switch]`, then passed after changing it to `[AK Switch]`.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml panic_hook::tests -- --nocapture && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after the panic hook branding update, 3 panic hook tests plus Cargo check.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test --manifest-path src-tauri/Cargo.toml commands::vscode_plugins::tests -- --nocapture && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after deterministic VS Code extension detection update, Rust plugin module 9 tests plus Cargo check.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts -t "top-level repository badges"`: first failed on Star History/Funding links still targeting upstream `cc-switch`, then passed after updating README and `.github/FUNDING.yml`.
- `corepack pnpm exec prettier --check tests/config/akSwitchBranding.test.ts .github/FUNDING.yml README.md README_ZH.md && corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts -t "top-level repository badges" && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check && PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`: passed after Star History/Funding/comment branding cleanup.
- `corepack pnpm exec vitest run tests/config/akSwitchBranding.test.ts && corepack pnpm run typecheck`: passed after the branding cleanup, 1 file / 7 tests plus `tsc --noEmit`.

## Remaining Gaps

1. Add runtime/manual verification for the real VS Code Claude and Codex extensions.
2. Install or fixture-test real Kilo/OpenCode VS Code extensions before claiming end-to-end plugin behavior.
3. Continue broader branding cleanup in localized docs, release notes, sponsorship copy, and legacy comments without changing compatibility identifiers.
4. Audit updater signing/public key and final GitHub release repository owner before distributing an AK Switch package; current updater/About links use `farion1231/ak-switch` and need confirmation or a configurable release owner.

## Next Step

Run the Tauri app against real local config files in a disposable profile, switch a Claude provider and an OpenCode provider, then confirm the VS Code plugin panel sync/clear actions write only AK Switch managed fields and restore cleanly.
