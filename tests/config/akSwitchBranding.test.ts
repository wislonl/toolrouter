import { readFileSync } from "node:fs";
import { readdirSync } from "node:fs";

describe("AK Switch release branding", () => {
  test("release workflow publishes AK Switch artifact names and release copy", () => {
    const workflow = readFileSync(".github/workflows/release.yml", "utf8");

    expect(workflow).toContain("AK-Switch-$VERSION-Windows.msi");
    expect(workflow).toContain("AK-Switch-$VERSION-Windows-Portable.zip");
    expect(workflow).toContain("AK-Switch-${VERSION}-Linux-${ARCH}.AppImage");
    expect(workflow).toContain("name: AK Switch ${{ github.ref_name }}");
    expect(workflow).toContain("## AK Switch ${{ github.ref_name }}");
    expect(workflow).not.toContain("CC-Switch-$VERSION-Windows.msi");
    expect(workflow).not.toContain("name: CC Switch ${{ github.ref_name }}");
  });

  test("top-level readmes use AK Switch package names for manual downloads", () => {
    const englishReadme = readFileSync("README.md", "utf8");
    const chineseReadme = readFileSync("README_ZH.md", "utf8");

    expect(englishReadme).toContain("AK-Switch-v{version}-Windows.msi");
    expect(englishReadme).toContain("AK-Switch-v{version}-macOS.dmg");
    expect(englishReadme).toContain("AK-Switch-v{version}-Linux.AppImage");
    expect(chineseReadme).toContain("AK-Switch-v{版本号}-Windows.msi");
    expect(chineseReadme).toContain("AK-Switch-v{版本号}-macOS.dmg");
    expect(chineseReadme).toContain("AK-Switch-v{版本号}-Linux.AppImage");
  });

  test("top-level readmes describe AK Switch runtime scope and storage", () => {
    const englishReadme = readFileSync("README.md", "utf8");
    const chineseReadme = readFileSync("README_ZH.md", "utf8");

    expect(englishReadme).toContain("## Why AK Switch?");
    expect(englishReadme).toContain(
      "**AK Switch** gives you a single desktop app",
    );
    expect(englishReadme).toContain(
      "Claude Code, Codex, Gemini CLI, OpenCode, OpenClaw, and Hermes",
    );
    expect(englishReadme).toContain(
      "VS Code plugin integration for Claude, Codex, Kilo, and OpenCode",
    );
    expect(englishReadme).toContain("**Deep Link** (`akswitch://`)");
    expect(englishReadme).toContain("`~/.ak-switch/ak-switch.db`");

    expect(chineseReadme).toContain("## 为什么选择 AK Switch？");
    expect(chineseReadme).toContain("**AK Switch** 为你提供一个桌面应用");
    expect(chineseReadme).toContain(
      "Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw 和 Hermes",
    );
    expect(chineseReadme).toContain(
      "VS Code 的 Claude、Codex、Kilo 和 OpenCode 插件集成",
    );
    expect(chineseReadme).toContain("**Deep Link** (`akswitch://`)");
    expect(chineseReadme).toContain("`~/.ak-switch/ak-switch.db`");
  });

  test("package metadata describes the expanded AK Switch scope", () => {
    const packageJson = readFileSync("package.json", "utf8");
    const cargoToml = readFileSync("src-tauri/Cargo.toml", "utf8");

    const expectedDescription =
      "All-in-One Manager for Claude Code, Codex, Gemini CLI, OpenCode, OpenClaw, Hermes Agent & VS Code Plugins";

    expect(packageJson).toContain(`"description": "${expectedDescription}"`);
    expect(cargoToml).toContain(`description = "${expectedDescription}"`);
    expect(packageJson).not.toContain(
      "All-in-One Assistant for Claude Code, Codex & Gemini CLI",
    );
    expect(cargoToml).not.toContain(
      "All-in-One Assistant for Claude Code, Codex & Gemini CLI",
    );
  });

  test("security and issue templates point to AK Switch instead of upstream CC Switch", () => {
    const security = readFileSync("SECURITY.md", "utf8");
    const issueTemplateDir = ".github/ISSUE_TEMPLATE";
    const issueTemplates = readdirSync(issueTemplateDir)
      .filter((file) => file.endsWith(".yml") || file.endsWith(".yaml"))
      .map((file) => readFileSync(`${issueTemplateDir}/${file}`, "utf8"))
      .join("\n");

    expect(security).toContain(
      "Only the latest release of AK Switch receives security updates.",
    );
    expect(security).toContain("this AK Switch repository");
    expect(security).not.toContain("farion1231/cc-switch");
    expect(security).not.toContain("latest release of CC Switch");

    expect(issueTemplates).toContain("AK Switch Version / 版本号");
    expect(issueTemplates).toContain("Hermes");
    expect(issueTemplates).toContain("VS Code Plugin / VS Code 插件");
    expect(issueTemplates).not.toContain("farion1231/cc-switch");
    expect(issueTemplates).not.toContain("CC Switch Version");
    expect(issueTemplates).not.toContain("CC Switch 3.11.1");
  });

  test("repository workflows describe AK Switch review context", () => {
    const claudeWorkflow = readFileSync(".github/workflows/claude.yml", "utf8");

    expect(claudeWorkflow).toContain("You are reviewing PRs for AK Switch");
    expect(claudeWorkflow).toContain("OpenCode, OpenClaw, Hermes");
    expect(claudeWorkflow).toContain("VS Code plugin integrations");
    expect(claudeWorkflow).not.toContain("reviewing PRs for cc-switch");
  });

  test("top-level repository badges and funding links point at the AK Switch fork", () => {
    const englishReadme = readFileSync("README.md", "utf8");
    const chineseReadme = readFileSync("README_ZH.md", "utf8");
    const funding = readFileSync(".github/FUNDING.yml", "utf8");

    expect(englishReadme).toContain("repos=farion1231/ak-switch&type=Date");
    expect(englishReadme).toContain("#farion1231/ak-switch&Date");
    expect(chineseReadme).toContain("repos=farion1231/ak-switch&type=Date");
    expect(chineseReadme).toContain("#farion1231/ak-switch&Date");
    expect(funding).toContain("github.com/farion1231/ak-switch");

    expect(englishReadme).not.toContain("repos=farion1231/cc-switch&type=Date");
    expect(chineseReadme).not.toContain("repos=farion1231/cc-switch&type=Date");
    expect(funding).not.toContain("github.com/farion1231/cc-switch");
  });
});
