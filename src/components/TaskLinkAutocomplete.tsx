import { useEffect, useState, useRef, useCallback } from "react";
import { api } from "@/lib/api";
import type { Task } from "@/types";

interface Props {
  query: string;
  onSelect: (taskTitle: string) => void;
  onClose: () => void;
}

export function TaskLinkAutocomplete({ query, onSelect, onClose }: Props) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api.getTasks().then((all) => {
      if (!all) {
        setTasks([]);
        return;
      }
      const filtered = all
        .filter((t) => t.status !== "archived")
        .filter((t) => t.title.toLowerCase().includes(query.toLowerCase()));
      setTasks(filtered.slice(0, 8));
      setSelectedIndex(0);
    });
  }, [query]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (tasks.length === 0) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % tasks.length);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + tasks.length) % tasks.length);
      } else if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        onSelect(tasks[selectedIndex].title);
      } else if (e.key === "Escape") {
        onClose();
      }
    },
    [tasks, selectedIndex, onSelect, onClose]
  );

  if (tasks.length === 0) return null;

  return (
    <div
      ref={containerRef}
      className="absolute z-50 mt-1 w-64 rounded-md border bg-popover shadow-md"
      onKeyDown={handleKeyDown}
    >
      <div className="px-2 py-1 text-xs text-muted-foreground">选择任务</div>
      {tasks.map((task, index) => (
        <button
          key={task.id}
          className={`w-full px-2 py-1.5 text-left text-sm hover:bg-accent ${
            index === selectedIndex ? "bg-accent" : ""
          }`}
          onClick={() => onSelect(task.title)}
          onMouseEnter={() => setSelectedIndex(index)}
        >
          {task.title}
        </button>
      ))}
    </div>
  );
}
