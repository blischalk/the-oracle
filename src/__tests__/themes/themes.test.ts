import { describe, it, expect } from "vitest";
import { THEMES, suggestedThemeForSystem } from "../../themes";

describe("themes", () => {
  it("has 6 themes", () => {
    expect(THEMES).toHaveLength(6);
  });

  it("every theme has id, label, cssClass", () => {
    THEMES.forEach((t) => {
      expect(t.id).toBeTruthy();
      expect(t.label).toBeTruthy();
      expect(t.cssClass).toBeTruthy();
    });
  });

  it("suggests mork-borg theme for mork-borg system", () => {
    expect(suggestedThemeForSystem("mork-borg")).toBe("mork-borg");
  });

  it("suggests dungeon theme for cairn", () => {
    expect(suggestedThemeForSystem("cairn")).toBe("dungeon");
  });

  it("falls back to default for unknown system", () => {
    expect(suggestedThemeForSystem("unknown-rpg-system")).toBe("default");
  });
});
