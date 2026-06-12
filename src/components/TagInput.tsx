import { useEffect, useState, useRef } from "react";
import { api } from "@/lib/api";
import type { Tag } from "@/types";

interface Props {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function TagInput({ value, onChange, placeholder = "按标签筛选..." }: Props) {
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    api.getAllTags().then((tags) => tags && setAllTags(tags));
  }, []);

  const filtered = value.trim()
    ? allTags.filter((t) =>
        t.name.toLowerCase().includes(value.toLowerCase()) ||
        (t.displayName?.toLowerCase().includes(value.toLowerCase()) ?? false)
      )
    : [];

  return (
    <div className="relative">
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onFocus={() => setShowSuggestions(true)}
        onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
        placeholder={placeholder}
        className="text-sm border rounded px-2 py-1 bg-background w-40"
      />
      {showSuggestions && filtered.length > 0 && (
        <div className="absolute z-50 mt-1 w-40 rounded-md border bg-popover shadow-md">
          {filtered.map((tag) => (
            <button
              key={tag.id}
              className="w-full px-2 py-1.5 text-left text-sm hover:bg-accent"
              onMouseDown={() => {
                onChange(tag.name);
                setShowSuggestions(false);
              }}
            >
              {tag.displayName || tag.name} ({tag.usageCount})
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
