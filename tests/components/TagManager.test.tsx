import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";

const mockTags = [
  { id: 1, name: "工作", displayName: "工作", usageCount: 3 },
  { id: 2, name: "灵感", displayName: "灵感", usageCount: 1 },
];

vi.mock("@/lib/api", () => ({
  api: {
    getAllTags: vi.fn(() => Promise.resolve(mockTags)),
    getEntryTags: vi.fn(() => Promise.resolve([mockTags[0]])),
    addTagToEntry: vi.fn(() => Promise.resolve()),
    syncEntryTags: vi.fn(() => Promise.resolve()),
  },
}));

import { TagManager } from "@/components/TagManager";
import { api } from "@/lib/api";

describe("TagManager", () => {
  it("renders current entry tags and all tags", async () => {
    render(<TagManager date="2026-06-11" />);
    await waitFor(() => {
      expect(screen.getByText("工作")).toBeInTheDocument();
    });
    expect(screen.getByText("灵感 (1)")).toBeInTheDocument();
  });

  it("calls addTagToEntry when adding a new tag", async () => {
    render(<TagManager date="2026-06-11" />);
    const input = screen.getByPlaceholderText("添加标签...");
    fireEvent.change(input, { target: { value: "新标签" } });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter" });
    await waitFor(() => {
      expect(api.addTagToEntry).toHaveBeenCalledWith("2026-06-11", "新标签");
    });
  });
});
