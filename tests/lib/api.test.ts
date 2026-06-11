import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args: Record<string, unknown>) => {
    if (cmd === "get_entry" && args.date === "2026-06-11") {
      return Promise.resolve({
        id: 1,
        date: "2026-06-11",
        content: "test",
        created_at: "",
        updated_at: "",
      });
    }
    if (cmd === "save_entry") {
      return Promise.resolve({
        id: 1,
        date: args.date,
        content: args.content,
        created_at: "",
        updated_at: "",
      });
    }
    return Promise.resolve(null);
  }),
}));

import { api } from "../../src/lib/api";

describe("api", () => {
  it("getEntry returns entry for date", async () => {
    const entry = await api.getEntry("2026-06-11");
    expect(entry).not.toBeNull();
    expect(entry?.date).toBe("2026-06-11");
  });

  it("saveEntry returns saved entry", async () => {
    const entry = await api.saveEntry("2026-06-11", "hello");
    expect(entry.content).toBe("hello");
  });
});
