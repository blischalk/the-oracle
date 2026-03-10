import { describe, it, expect } from "vitest";
import { THEMES, DEFAULT_THEME_ID } from "../../themes";

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

  it("default theme id points to a real theme", () => {
    expect(THEMES.find((t) => t.id === DEFAULT_THEME_ID)).toBeDefined();
  });

  it("css class follows theme-<id> convention", () => {
    THEMES.forEach((t) => {
      expect(t.cssClass).toBe(`theme-${t.id}`);
    });
  });
});
