import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import type { Tag } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";

interface TagManagerProps {
  date: string;
  onTagClick?: (tagName: string) => void;
}

export function TagManager({ date, onTagClick }: TagManagerProps) {
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [entryTags, setEntryTags] = useState<Tag[]>([]);
  const [newTag, setNewTag] = useState("");

  const loadTags = async () => {
    const [all, current] = await Promise.all([
      api.getAllTags(),
      api.getEntryTags(date),
    ]);
    if (all) setAllTags(all);
    if (current) setEntryTags(current);
  };

  useEffect(() => {
    loadTags();
  }, [date]);

  const handleAdd = async () => {
    const trimmed = newTag.trim();
    if (!trimmed) return;
    await api.addTagToEntry(date, trimmed);
    setNewTag("");
    await loadTags();
  };

  const handleRemove = async (tagName: string) => {
    // For v0.2.0, re-sync entry tags from content minus the removed tag.
    // Since there's no direct remove command, we rebuild the tag list.
    const remaining = entryTags
      .filter((t) => t.name !== tagName)
      .map((t) => `#${t.displayName || t.name}`)
      .join(" ");
    await api.syncEntryTags(date, remaining);
    await loadTags();
  };

  const isExisting = (name: string) =>
    entryTags.some((t) => t.name === name.toLowerCase());

  return (
    <div className="space-y-3 p-3 border rounded-lg bg-card">
      <h4 className="text-sm font-medium">标签</h4>
      {entryTags.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {entryTags.map((tag) => (
            <Badge
              key={tag.id}
              variant="secondary"
              className="cursor-pointer gap-1 pr-1"
              onClick={() => onTagClick?.(tag.name)}
            >
              {tag.displayName || tag.name}
              <span
                role="button"
                aria-label={`移除 ${tag.name}`}
                onClick={(e) => {
                  e.stopPropagation();
                  handleRemove(tag.name);
                }}
                className="rounded-full hover:bg-secondary-foreground/20 px-1"
              >
                ×
              </span>
            </Badge>
          ))}
        </div>
      )}
      <div className="flex gap-2">
        <Input
          value={newTag}
          onChange={(e) => setNewTag(e.target.value)}
          placeholder="添加标签..."
          className="h-8 text-sm"
          onKeyDown={(e) => e.key === "Enter" && handleAdd()}
        />
      </div>
      {allTags.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {allTags
            .filter((tag) => !isExisting(tag.name))
            .map((tag) => (
              <Badge
                key={tag.id}
                variant="outline"
                className="cursor-pointer text-xs"
                onClick={() => onTagClick?.(tag.name)}
              >
                {tag.displayName || tag.name} ({tag.usageCount})
              </Badge>
            ))}
        </div>
      )}
    </div>
  );
}
