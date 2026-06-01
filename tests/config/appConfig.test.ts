import { APP_IDS, MCP_APP_IDS, SKILLS_APP_IDS } from "@/config/appConfig";

describe("app configuration", () => {
  it("exposes OpenClaw and Hermes in the global app list", () => {
    expect(APP_IDS).toEqual(expect.arrayContaining(["openclaw", "hermes"]));
  });

  it("allows Skills for OpenClaw and Hermes while keeping OpenClaw out of MCP", () => {
    expect(SKILLS_APP_IDS).toEqual(
      expect.arrayContaining(["openclaw", "hermes"]),
    );
    expect(MCP_APP_IDS).toContain("hermes");
    expect(MCP_APP_IDS).not.toContain("openclaw");
  });
});
