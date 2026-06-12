import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";

const mockTasks = [
  { id: 1, title: "完成设计", description: "", quadrant: 1, status: "active", created_at: "", completed_at: null, updated_at: "" },
  { id: 2, title: "需求评审", description: "", quadrant: 2, status: "active", created_at: "", completed_at: null, updated_at: "" },
];

vi.mock("@/lib/api", () => ({
  api: {
    getTasks: vi.fn(() => Promise.resolve(mockTasks)),
  },
}));

import { TaskLinkAutocomplete } from "@/components/TaskLinkAutocomplete";

describe("TaskLinkAutocomplete", () => {
  it("renders filtered tasks", async () => {
    render(<TaskLinkAutocomplete query="设计" onSelect={vi.fn()} onClose={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText("完成设计")).toBeInTheDocument();
    });
    expect(screen.queryByText("需求评审")).not.toBeInTheDocument();
  });

  it("calls onSelect when clicking a task", async () => {
    const onSelect = vi.fn();
    render(<TaskLinkAutocomplete query="设计" onSelect={onSelect} onClose={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText("完成设计")).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText("完成设计"));
    expect(onSelect).toHaveBeenCalledWith("完成设计");
  });

  it("supports keyboard navigation", async () => {
    const onSelect = vi.fn();
    render(<TaskLinkAutocomplete query="" onSelect={onSelect} onClose={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText("完成设计")).toBeInTheDocument();
    });
    fireEvent.keyDown(screen.getByText("完成设计"), { key: "ArrowDown" });
    fireEvent.keyDown(screen.getByText("需求评审"), { key: "Enter" });
    expect(onSelect).toHaveBeenCalledWith("需求评审");
  });
});
