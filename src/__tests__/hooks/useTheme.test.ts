import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));

const { useTheme } = await import("../../hooks/useTheme");

describe("useTheme", () => {
  it("returns themes list", () => {
    const { result } = renderHook(() => useTheme());
    expect(result.current.themes.length).toBeGreaterThan(0);
  });

  it("changes theme", async () => {
    const { result } = renderHook(() => useTheme());
    act(() => {
      result.current.setTheme("ember");
    });
    expect(result.current.activeTheme).toBe("ember");
  });
});
