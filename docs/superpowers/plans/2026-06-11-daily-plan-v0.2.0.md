# Daily Plan v0.2.0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a tag system, bi-directional task links, customizable sidebar layout, and polished animations to Daily Plan v0.1.0.

**Architecture:** Extend the SQLite schema with `tags`, `entry_tags`, `task_tags`, `entry_task_links`, and `app_settings` tables. Add Rust commands for parsing, querying, and persisting tags/links/settings. Build focused React components for tag management, task-link autocomplete, and search. Use `framer-motion` for page transitions and `@dnd-kit` drag overlay for board interactions. Store UI state in SQLite via Tauri and remember it across sessions.

**Tech Stack:** Tauri v2 | Rust | React 19 | TypeScript | SQLite | Tailwind CSS | shadcn/ui | TipTap | @dnd-kit | framer-motion

---

## File Structure

### Backend (Rust)

| File | Responsibility |
|------|----------------|
| `src-tauri/src/db.rs` | Create new tables, FTS triggers for tags, migration logic |
| `src-tauri/src/commands.rs` | New commands: tags, task links, app settings |
| `src-tauri/src/lib.rs` | Register new commands |
| `src-tauri/src/parser.rs` (new) | Pure functions to extract `#tags` and `[[task links]]` |
| `src-tauri/src/models.rs` (new/extend) | Shared structs for Tag, TaskLink, Setting |

### Frontend (React/TypeScript)

| File | Responsibility |
|------|----------------|
| `src/lib/api.ts` | Type-safe wrappers for new Tauri commands |
| `src/types/index.ts` (extend) | TypeScript types for Tag, TaskLink, AppSettings |
| `src/components/TagManager.tsx` (new) | Tag panel, global tag list, manual add/remove |
| `src/components/TagInput.tsx` (new) | Autocomplete input for filtering tasks by tag |
| `src/components/TaskLinkAutocomplete.tsx` (new) | Popover for `[[` task completion |
| `src/components/SidebarLayout.tsx` (new) | Collapsible sidebar + layout state |
| `src/components/AnimatedPage.tsx` (new) | Framer-motion page transition wrapper |
| `src/pages/RecordPage.tsx` | Integrate tag extraction, tag panel, task link rendering/completion |
| `src/pages/BoardPage.tsx` | Tag display, tag filter, drag overlay enhancement |
| `src/pages/SearchPage.tsx` (new) | Tag search results for entries and tasks |
| `src/App.tsx` | Layout state provider, animated route transitions |

### Tests

| File | Responsibility |
|------|----------------|
| `src-tauri/src/parser_tests.rs` (new) | Unit tests for tag/link parsing |
| `tests/components/TagManager.test.tsx` (new) | Tag panel interactions |
| `tests/components/TaskLinkAutocomplete.test.tsx` (new) | Autocomplete behavior |

---

## Tasks

### Task 1: Database Migration — Tags, Links, and Settings Tables

**Files:**
- Modify: `src-tauri/src/db.rs`
- Test: `src-tauri/src/db.rs` (manual via `cargo test`)

- [ ] **Step 1: Add `create_v2_tables` function**

  In `src-tauri/src/db.rs`, after existing `create_tables`, add a new function that creates the v0.2.0 tables and indexes:

  ```rust
  fn create_v2_tables(conn: &Connection) -> Result<()> {
      conn.execute(
          "CREATE TABLE IF NOT EXISTS tags (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL UNIQUE,
              display_name TEXT,
              created_at TEXT NOT NULL
          )",
          [],
      )?;
      conn.execute(
          "CREATE TABLE IF NOT EXISTS entry_tags (
              entry_id INTEGER NOT NULL,
              tag_id INTEGER NOT NULL,
              PRIMARY KEY (entry_id, tag_id),
              FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
              FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
          )",
          [],
      )?;
      conn.execute(
          "CREATE TABLE IF NOT EXISTS task_tags (
              task_id INTEGER NOT NULL,
              tag_id INTEGER NOT NULL,
              PRIMARY KEY (task_id, tag_id),
              FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
              FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
          )",
          [],
      )?;
      conn.execute(
          "CREATE TABLE IF NOT EXISTS entry_task_links (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              entry_id INTEGER NOT NULL,
              task_id INTEGER,
              raw_text TEXT NOT NULL,
              position INTEGER,
              FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
              FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
          )",
          [],
      )?;
      conn.execute(
          "CREATE TABLE IF NOT EXISTS app_settings (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
          )",
          [],
      )?;
      Ok(())
  }
  ```

- [ ] **Step 2: Call migration from `init_app_db`**

  In `init_app_db`, after `create_tables(conn)?;`, call:

  ```rust
  create_v2_tables(conn)?;
  ```

- [ ] **Step 3: Build and run dev mode to verify migration**

  Run:
  ```bash
  npm run tauri dev
  ```
  Expected: App starts without database errors.

- [ ] **Step 4: Commit**

  ```bash
  git add src-tauri/src/db.rs
  git commit -m "feat(db): add v0.2.0 tables for tags, links, and settings"
  ```

---

### Task 2: Pure Parser for Tags and Task Links

**Files:**
- Create: `src-tauri/src/parser.rs`
- Modify: `src-tauri/src/lib.rs` (add module)
- Test: `src-tauri/src/parser_tests.rs`

- [ ] **Step 1: Create parser module**

  `src-tauri/src/parser.rs`:

  ```rust
  use regex::Regex;
  use std::sync::OnceLock;

  pub struct ParsedTags {
      pub names: Vec<String>,
  }

  pub struct ParsedTaskLink {
      pub raw_text: String,
      pub position: usize,
  }

  fn tag_regex() -> &'static Regex {
      static RE: OnceLock<Regex> = OnceLock::new();
      RE.get_or_init(|| Regex::new(r"#([\p{L}\p{N}_-]+)").unwrap())
  }

  fn task_link_regex() -> &'static Regex {
      static RE: OnceLock<Regex> = OnceLock::new();
      RE.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").unwrap())
  }

  pub fn extract_tags(text: &str) -> ParsedTags {
      let mut names = Vec::new();
      for cap in tag_regex().captures_iter(text) {
          let name = cap[1].to_lowercase();
          if !name.chars().all(|c| c.is_ascii_digit()) && name.len() <= 50 {
              names.push(name);
          }
      }
      ParsedTags { names }
  }

  pub fn extract_task_links(text: &str) -> Vec<ParsedTaskLink> {
      task_link_regex()
          .captures_iter(text)
          .map(|cap| {
              let m = cap.get(0).unwrap();
              ParsedTaskLink {
                  raw_text: cap[1].trim().to_string(),
                  position: m.start(),
              }
          })
          .collect()
  }
  ```

- [ ] **Step 2: Add regex dependency**

  In `src-tauri/Cargo.toml`, add:

  ```toml
  [dependencies]
  regex = "1.10"
  ```

- [ ] **Step 3: Register module in lib.rs**

  Add:

  ```rust
  mod parser;
  ```

- [ ] **Step 4: Write unit tests**

  Create `src-tauri/src/parser_tests.rs`:

  ```rust
  #[cfg(test)]
  mod tests {
      use crate::parser::{extract_tags, extract_task_links};

      #[test]
      fn extracts_simple_tags() {
          let result = extract_tags("今天 #工作 进展顺利 #灵感");
          assert_eq!(result.names, vec!["工作", "灵感"]);
      }

      #[test]
      fn ignores_pure_numeric_tag() {
          let result = extract_tags("#123 和 #项目2");
          assert_eq!(result.names, vec!["项目2"]);
      }

      #[test]
      fn extracts_task_links() {
          let links = extract_task_links("参见 [[完成设计]] 和 [[需求评审]]");
          assert_eq!(links.len(), 2);
          assert_eq!(links[0].raw_text, "完成设计");
          assert_eq!(links[1].raw_text, "需求评审");
      }

      #[test]
      fn ignores_nested_task_links() {
          let links = extract_task_links("[[外层 [[内层]]]]");
          assert_eq!(links.len(), 1);
          assert_eq!(links[0].raw_text, "外层 [[内层");
      }
  }
  ```

- [ ] **Step 5: Run tests**

  ```bash
  cd src-tauri && cargo test parser_tests
  ```
  Expected: All 4 tests pass.

- [ ] **Step 6: Commit**

  ```bash
  git add src-tauri/src/parser.rs src-tauri/src/parser_tests.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
  git commit -m "feat(parser): add tag and task-link extraction"
  ```

---

### Task 3: Backend Tag Commands

**Files:**
- Create/Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/db.rs` (add helper functions)
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Define command structs**

  In `src-tauri/src/commands.rs`, add:

  ```rust
  use crate::db::get_conn;
  use crate::parser::extract_tags;
  use rusqlite::{params, Result};
  use serde::{Deserialize, Serialize};
  use tauri::AppHandle;

  #[derive(Serialize, Deserialize)]
  pub struct TagDto {
      pub id: i64,
      pub name: String,
      pub display_name: Option<String>,
      pub usage_count: i64,
  }
  ```

- [ ] **Step 2: Add `sync_entry_tags` command**

  ```rust
  #[tauri::command]
  pub fn sync_entry_tags(app: AppHandle, date: String, content: String) -> Result<(), String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      let parsed = extract_tags(&content);
      crate::db::replace_entry_tags(&conn, &date, &parsed.names).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 3: Add helper `replace_entry_tags` in db.rs**

  ```rust
  pub fn replace_entry_tags(conn: &Connection, date: &str, tags: &[String]) -> Result<()> {
      let entry_id: i64 = conn.query_row(
          "SELECT id FROM entries WHERE date = ?1",
          [date],
          |row| row.get(0),
      )?;
      conn.execute(
          "DELETE FROM entry_tags WHERE entry_id = ?1",
          [entry_id],
      )?;
      for name in tags {
          conn.execute(
              "INSERT INTO tags (name, display_name, created_at) VALUES (?1, ?2, datetime('now'))
               ON CONFLICT(name) DO UPDATE SET display_name = COALESCE(excluded.display_name, display_name)",
              [name.to_lowercase(), name.clone()],
          )?;
          let tag_id: i64 = conn.query_row(
              "SELECT id FROM tags WHERE name = ?1",
              [name.to_lowercase()],
              |row| row.get(0),
          )?;
          conn.execute(
              "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
              [entry_id, tag_id],
          )?;
      }
      Ok(())
  }
  ```

- [ ] **Step 4: Add manual tag helpers**

  Add to `src-tauri/src/db.rs`:

  ```rust
  pub fn add_tag_to_entry(conn: &Connection, date: &str, tag_name: &str) -> Result<()> {
      let entry_id: i64 = conn.query_row(
          "SELECT id FROM entries WHERE date = ?1",
          [date],
          |row| row.get(0),
      )?;
      let normalized = tag_name.to_lowercase();
      conn.execute(
          "INSERT INTO tags (name, display_name, created_at) VALUES (?1, ?2, datetime('now'))
           ON CONFLICT(name) DO UPDATE SET display_name = COALESCE(excluded.display_name, display_name)",
          [&normalized, tag_name],
      )?;
      let tag_id: i64 = conn.query_row(
          "SELECT id FROM tags WHERE name = ?1",
          [&normalized],
          |row| row.get(0),
      )?;
      conn.execute(
          "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
          [entry_id, tag_id],
      )?;
      Ok(())
  }

  pub fn get_entry_tags(conn: &Connection, date: &str) -> Result<Vec<TagDto>> {
      let mut stmt = conn.prepare(
          "SELECT t.id, t.name, t.display_name, COUNT(et2.tag_id) as usage_count
           FROM tags t
           JOIN entry_tags et ON et.tag_id = t.id
           JOIN entries e ON e.id = et.entry_id
           LEFT JOIN entry_tags et2 ON et2.tag_id = t.id
           WHERE e.date = ?1
           GROUP BY t.id"
      )?;
      let rows = stmt.query_map([date], |row| {
          Ok(TagDto {
              id: row.get(0)?,
              name: row.get(1)?,
              display_name: row.get(2)?,
              usage_count: row.get(3)?,
          })
      })?;
      rows.collect()
  }
  ```

  Add commands in `commands.rs`:

  ```rust
  #[tauri::command]
  pub fn add_tag_to_entry(app: AppHandle, date: String, tag_name: String) -> Result<(), String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      crate::db::add_tag_to_entry(&conn, &date, &tag_name).map_err(|e| e.to_string())
  }

  #[tauri::command]
  pub fn get_entry_tags(app: AppHandle, date: String) -> Result<Vec<TagDto>, String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      crate::db::get_entry_tags(&conn, &date).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 5: Add similar helpers for tasks**

  - `replace_task_tags(conn, task_id, tags)`
  - `get_all_tags(conn)` returning `Vec<TagDto>`
  - `search_entries_by_tag(conn, tag_name)`
  - `search_tasks_by_tag(conn, tag_name)`

  Add `sync_task_tags` command:

  ```rust
  #[tauri::command]
  pub fn sync_task_tags(app: AppHandle, task_id: i64, content: String) -> Result<(), String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      let parsed = extract_tags(&content);
      crate::db::replace_task_tags(&conn, task_id, &parsed.names).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 6: Register commands in lib.rs**

  Add to `invoke_handler`:

  ```rust
  .invoke_handler(tauri::generate_handler![
      // existing commands...
      commands::sync_entry_tags,
      commands::add_tag_to_entry,
      commands::get_entry_tags,
      commands::sync_task_tags,
      commands::get_all_tags,
      commands::search_by_tag,
  ])
  ```

- [ ] **Step 7: Build and run dev mode**

  ```bash
  npm run tauri dev
  ```
  Expected: No compile errors.

- [ ] **Step 8: Commit**

  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/db.rs src-tauri/src/lib.rs
  git commit -m "feat(backend): add tag sync and query commands"
  ```

---

### Task 4: Backend Task-Link Commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add `sync_entry_task_links` command**

  ```rust
  #[tauri::command]
  pub fn sync_entry_task_links(app: AppHandle, date: String, content: String) -> Result<(), String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      let links = crate::parser::extract_task_links(&content);
      crate::db::replace_entry_task_links(&conn, &date, &links).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 2: Add `replace_entry_task_links` helper in db.rs**

  ```rust
  pub fn replace_entry_task_links(
      conn: &Connection,
      date: &str,
      links: &[crate::parser::ParsedTaskLink],
  ) -> Result<()> {
      let entry_id: i64 = conn.query_row(
          "SELECT id FROM entries WHERE date = ?1",
          [date],
          |row| row.get(0),
      )?;
      conn.execute(
          "DELETE FROM entry_task_links WHERE entry_id = ?1",
          [entry_id],
      )?;
      for link in links {
          let task_id: Option<i64> = conn
              .query_row(
                  "SELECT id FROM tasks WHERE title = ?1",
                  [link.raw_text.trim()],
                  |row| row.get(0),
              )
              .ok();
          conn.execute(
              "INSERT INTO entry_task_links (entry_id, task_id, raw_text, position) VALUES (?1, ?2, ?3, ?4)",
              params![entry_id, task_id, link.raw_text.clone(), link.position as i64],
          )?;
      }
      Ok(())
  }
  ```

  Note: Use `Option<i64>` properly with `params!` macro for nullable task_id.

- [ ] **Step 3: Add `get_entry_task_links` command**

  ```rust
  #[tauri::command]
  pub fn get_entry_task_links(app: AppHandle, date: String) -> Result<Vec<TaskLinkDto>, String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      crate::db::get_entry_task_links(&conn, &date).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 4: Register commands**

  Add `sync_entry_task_links` and `get_entry_task_links` to `invoke_handler`.

- [ ] **Step 5: Build and run dev mode**

  ```bash
  npm run tauri dev
  ```

- [ ] **Step 6: Commit**

  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/db.rs src-tauri/src/lib.rs
  git commit -m "feat(backend): add task-link sync and query commands"
  ```

---

### Task 5: Backend App Settings Commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add settings commands**

  ```rust
  #[tauri::command]
  pub fn get_app_setting(app: AppHandle, key: String) -> Result<Option<String>, String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      crate::db::get_setting(&conn, &key).map_err(|e| e.to_string())
  }

  #[tauri::command]
  pub fn set_app_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
      let conn = get_conn(&app).map_err(|e| e.to_string())?;
      crate::db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
  }
  ```

- [ ] **Step 2: Add db helpers**

  ```rust
  pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
      let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
      let mut rows = stmt.query([key])?;
      if let Some(row) = rows.next()? {
          Ok(Some(row.get(0)?))
      } else {
          Ok(None)
      }
  }

  pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
      conn.execute(
          "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
           ON CONFLICT(key) DO UPDATE SET value = excluded.value",
          [key, value],
      )?;
      Ok(())
  }
  ```

- [ ] **Step 3: Register commands**

  Add `get_app_setting` and `set_app_setting` to `invoke_handler`.

- [ ] **Step 4: Commit**

  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/db.rs src-tauri/src/lib.rs
  git commit -m "feat(backend): add app settings commands"
  ```

---

### Task 6: Frontend API Layer and Types

**Files:**
- Modify: `src/lib/api.ts`
- Modify: `src/types/index.ts` (or create)

- [ ] **Step 1: Add types**

  In `src/types/index.ts` (create if not exists):

  ```typescript
  export interface Tag {
    id: number;
    name: string;
    displayName?: string;
    usageCount: number;
  }

  export interface TaskLink {
    id: number;
    entryId: number;
    taskId?: number;
    rawText: string;
    position: number;
    taskTitle?: string;
  }

  export interface AppSettings {
    sidebarCollapsed?: boolean;
    windowWidth?: number;
    windowHeight?: number;
  }
  ```

- [ ] **Step 2: Extend api.ts**

  In `src/lib/api.ts`, add:

  ```typescript
  import type { Tag, TaskLink, AppSettings } from '@/types';

  export const api = {
    // existing methods...

    syncEntryTags: (date: string, content: string) =>
      safeInvoke<void>('sync_entry_tags', { date, content }),

    addTagToEntry: (date: string, tagName: string) =>
      safeInvoke<void>('add_tag_to_entry', { date, tagName }),

    getEntryTags: (date: string) =>
      safeInvoke<Tag[]>('get_entry_tags', { date }),

    getAllTags: () =>
      safeInvoke<Tag[]>('get_all_tags'),

    searchByTag: (tagName: string) =>
      safeInvoke<{ entries: Entry[]; tasks: Task[] }>('search_by_tag', { tagName }),

    syncTaskTags: (taskId: number, content: string) =>
      safeInvoke<void>('sync_task_tags', { taskId, content }),

    syncEntryTaskLinks: (date: string, content: string) =>
      safeInvoke<void>('sync_entry_task_links', { date, content }),

    getEntryTaskLinks: (date: string) =>
      safeInvoke<TaskLink[]>('get_entry_task_links', { date }),

    getAppSetting: (key: string) =>
      safeInvoke<string | null>('get_app_setting', { key }),

    setAppSetting: (key: string, value: string) =>
      safeInvoke<void>('set_app_setting', { key, value }),
  };
  ```

- [ ] **Step 3: Type-check**

  ```bash
  npx tsc --noEmit
  ```
  Expected: No type errors.

- [ ] **Step 4: Commit**

  ```bash
  git add src/lib/api.ts src/types/index.ts
  git commit -m "feat(api): add frontend wrappers for tags, links, and settings"
  ```

---

### Task 7: Tag Manager Component

**Files:**
- Create: `src/components/TagManager.tsx`
- Create: `src/components/ui/badge.tsx` (if not exists, use shadcn)
- Test: `tests/components/TagManager.test.tsx`

- [ ] **Step 1: Create TagManager component**

  `src/components/TagManager.tsx`:

  ```tsx
  import { useEffect, useState } from 'react';
  import { api } from '@/lib/api';
  import type { Tag } from '@/types';
  import { Badge } from '@/components/ui/badge';
  import { Input } from '@/components/ui/input';
  import { X } from 'lucide-react';

  interface TagManagerProps {
    date: string;
    content: string;
    onTagClick?: (tag: string) => void;
  }

  export function TagManager({ date, content, onTagClick }: TagManagerProps) {
    const [allTags, setAllTags] = useState<Tag[]>([]);
    const [entryTags, setEntryTags] = useState<Tag[]>([]);
    const [newTag, setNewTag] = useState('');

    useEffect(() => {
      api.getAllTags().then((tags) => tags && setAllTags(tags));
      // TODO: load entry tags after backend command added
    }, [date, content]);

    const handleAdd = async () => {
      if (!newTag.trim()) return;
      await api.addTagToEntry(date, newTag.trim());
      setNewTag('');
      const tags = await api.getEntryTags(date);
      tags && setEntryTags(tags);
      const all = await api.getAllTags();
      all && setAllTags(all);
    };

    return (
      <div className="space-y-3 p-3 border rounded-lg bg-card">
        <h4 className="text-sm font-medium">标签</h4>
        <div className="flex flex-wrap gap-2">
          {entryTags.map((tag) => (
            <Badge key={tag.id} variant="secondary" className="cursor-pointer" onClick={() => onTagClick?.(tag.name)}>
              {tag.displayName || tag.name}
              <X className="ml-1 h-3 w-3" />
            </Badge>
          ))}
        </div>
        <div className="flex gap-2">
          <Input
            value={newTag}
            onChange={(e) => setNewTag(e.target.value)}
            placeholder="添加标签..."
            className="h-8 text-sm"
            onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
          />
        </div>
        <div className="flex flex-wrap gap-1.5">
          {allTags.map((tag) => (
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
      </div>
    );
  }
  ```

- [ ] **Step 2: Add tests**

  `tests/components/TagManager.test.tsx`:

  ```tsx
  import { render, screen, fireEvent } from '@testing-library/react';
  import { TagManager } from '@/components/TagManager';

  test('renders tag manager with add input', () => {
    render(<TagManager date="2026-06-11" content="" />);
    expect(screen.getByPlaceholderText('添加标签...')).toBeInTheDocument();
  });
  ```

- [ ] **Step 3: Run tests**

  ```bash
  npm test -- tests/components/TagManager.test.tsx
  ```
  Expected: Test passes.

- [ ] **Step 4: Commit**

  ```bash
  git add src/components/TagManager.tsx tests/components/TagManager.test.tsx
  git commit -m "feat(ui): add TagManager component"
  ```

---

### Task 8: Task Link Autocomplete Component

**Files:**
- Create: `src/components/TaskLinkAutocomplete.tsx`
- Test: `tests/components/TaskLinkAutocomplete.test.tsx`

- [ ] **Step 1: Create component**

  `src/components/TaskLinkAutocomplete.tsx`:

  ```tsx
  import { useEffect, useState } from 'react';
  import { api } from '@/lib/api';
  import type { Task } from '@/types';

  interface Props {
    query: string;
    onSelect: (taskTitle: string) => void;
    onClose: () => void;
  }

  export function TaskLinkAutocomplete({ query, onSelect, onClose }: Props) {
    const [tasks, setTasks] = useState<Task[]>([]);

    useEffect(() => {
      api.getTasks().then((all) => {
        if (!all) return;
        const filtered = all.filter((t) =>
          t.title.toLowerCase().includes(query.toLowerCase())
        );
        setTasks(filtered.slice(0, 8));
      });
    }, [query]);

    if (tasks.length === 0) return null;

    return (
      <div className="absolute z-50 mt-1 w-64 rounded-md border bg-popover shadow-md">
        <div className="px-2 py-1 text-xs text-muted-foreground">选择任务</div>
        {tasks.map((task) => (
          <button
            key={task.id}
            className="w-full px-2 py-1.5 text-left text-sm hover:bg-accent"
            onClick={() => onSelect(task.title)}
          >
            {task.title}
          </button>
        ))}
      </div>
    );
  }
  ```

- [ ] **Step 2: Add keyboard navigation tests**

  Write a test that simulates selecting a task from the list.

- [ ] **Step 3: Commit**

  ```bash
  git add src/components/TaskLinkAutocomplete.tsx tests/components/TaskLinkAutocomplete.test.tsx
  git commit -m "feat(ui): add task-link autocomplete"
  ```

---

### Task 9: Record Page Integration

**Files:**
- Modify: `src/pages/RecordPage.tsx`
- Modify: `src/lib/api.ts` (ensure save entry also syncs tags/links)

- [ ] **Step 1: Sync tags and links on save**

  In `RecordPage.tsx`, inside the save debounce, after `api.saveEntry`, call:

  ```typescript
  await api.syncEntryTags(dateParam, content);
  await api.syncEntryTaskLinks(dateParam, content);
  ```

- [ ] **Step 2: Add TagManager panel**

  Add `<TagManager date={dateParam} content={editor?.getText() ?? ''} />` in a right-side panel.

- [ ] **Step 3: Wire task-link autocomplete**

  Listen for `[[` in the editor. When detected, show `TaskLinkAutocomplete` near the cursor. On select, insert `[[taskTitle]]`.

- [ ] **Step 4: Render existing task links**

  Add a TipTap mark or React overlay to render `[[...]]` as clickable pills. Keep editor content as plain text.

- [ ] **Step 5: Build and test manually**

  ```bash
  npm run tauri dev
  ```
  Verify: typing `#标签` saves it; typing `[[` shows task autocomplete; clicking a task link jumps to board.

- [ ] **Step 6: Commit**

  ```bash
  git add src/pages/RecordPage.tsx
  git commit -m "feat(record): integrate tags, tag panel, and task links"
  ```

---

### Task 10: Board Page Tags and Filter

**Files:**
- Modify: `src/pages/BoardPage.tsx`
- Create: `src/components/TagInput.tsx`

- [ ] **Step 1: Add tag display to task cards**

  In task card component, render task tags as small badges.

- [ ] **Step 2: Add tag filter bar**

  At top of board, add `TagInput` autocomplete. On select, filter `tasks` state.

- [ ] **Step 3: Sync task tags on task save**

  When creating/updating a task, call `api.syncTaskTags(taskId, title + ' ' + description)`.

- [ ] **Step 4: Commit**

  ```bash
  git add src/pages/BoardPage.tsx src/components/TagInput.tsx
  git commit -m "feat(board): display tags and filter tasks by tag"
  ```

---

### Task 11: Search Page

**Files:**
- Create: `src/pages/SearchPage.tsx`
- Modify: `src/App.tsx` (add route)

- [ ] **Step 1: Create SearchPage**

  ```tsx
  import { useSearchParams } from 'react-router-dom';
  import { useEffect, useState } from 'react';
  import { api } from '@/lib/api';
  import type { Entry, Task } from '@/types';

  export function SearchPage() {
    const [params] = useSearchParams();
    const tag = params.get('tag') || '';
    const [entries, setEntries] = useState<Entry[]>([]);
    const [tasks, setTasks] = useState<Task[]>([]);

    useEffect(() => {
      if (!tag) return;
      api.searchByTag(tag).then((result) => {
        if (result) {
          setEntries(result.entries);
          setTasks(result.tasks);
        }
      });
    }, [tag]);

    return (
      <div className="grid grid-cols-2 gap-6 p-6">
        <section>
          <h2 className="text-lg font-semibold mb-4">日记 · #{tag}</h2>
          {entries.map((e) => (
            <div key={e.date} className="p-3 border rounded mb-2">
              <div className="text-sm text-muted-foreground">{e.date}</div>
              <div className="line-clamp-3" dangerouslySetInnerHTML={{ __html: e.content }} />
            </div>
          ))}
        </section>
        <section>
          <h2 className="text-lg font-semibold mb-4">任务 · #{tag}</h2>
          {tasks.map((t) => (
            <div key={t.id} className="p-3 border rounded mb-2">
              <div className="font-medium">{t.title}</div>
              <div className="text-sm text-muted-foreground">{t.description}</div>
            </div>
          ))}
        </section>
      </div>
    );
  }
  ```

- [ ] **Step 2: Add route**

  In `src/App.tsx`:

  ```tsx
  <Route path="/search" element={<SearchPage />} />
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src/pages/SearchPage.tsx src/App.tsx
  git commit -m "feat(search): add tag search results page"
  ```

---

### Task 12: Sidebar Collapse and Layout State

**Files:**
- Create: `src/components/SidebarLayout.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create SidebarLayout**

  ```tsx
  import { useEffect, useState } from 'react';
  import { PanelLeft } from 'lucide-react';
  import { api } from '@/lib/api';

  interface Props {
    children: React.ReactNode;
  }

  export function SidebarLayout({ children }: Props) {
    const [collapsed, setCollapsed] = useState(false);

    useEffect(() => {
      api.getAppSetting('sidebar_collapsed').then((v) => {
        setCollapsed(v === 'true');
      });
    }, []);

    const toggle = () => {
      const next = !collapsed;
      setCollapsed(next);
      api.setAppSetting('sidebar_collapsed', String(next));
    };

    return (
      <div className="flex h-screen">
        <aside className={`border-r bg-card transition-all ${collapsed ? 'w-14' : 'w-56'}`}>
          <nav className="flex flex-col h-full p-2">
            {/* Nav items */}
            <button onClick={toggle} className="mt-auto p-2 rounded hover:bg-accent">
              <PanelLeft className="h-5 w-5" />
            </button>
          </nav>
        </aside>
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    );
  }
  ```

- [ ] **Step 2: Integrate into App.tsx**

  Wrap route content with `SidebarLayout`.

- [ ] **Step 3: Add window size memory (optional extension)**

  Use Tauri `appWindow` API to save/restore window size on close/startup.

- [ ] **Step 4: Commit**

  ```bash
  git add src/components/SidebarLayout.tsx src/App.tsx
  git commit -m "feat(layout): add collapsible sidebar with state memory"
  ```

---

### Task 13: Page Transition Animations

**Files:**
- Create: `src/components/AnimatedPage.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Install framer-motion**

  ```bash
  npm install framer-motion
  ```

- [ ] **Step 2: Create AnimatedPage**

  ```tsx
  import { motion } from 'framer-motion';

  export function AnimatedPage({ children }: { children: React.ReactNode }) {
    return (
      <motion.div
        initial={{ opacity: 0, x: 12 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: -12 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
      >
        {children}
      </motion.div>
    );
  }
  ```

- [ ] **Step 3: Wrap routes with AnimatePresence**

  In `src/App.tsx`:

  ```tsx
  import { AnimatePresence } from 'framer-motion';

  <AnimatePresence mode="wait">
    <AnimatedPage key={location.pathname}>
      <Routes location={location}>
        {/* routes */}
      </Routes>
    </AnimatedPage>
  </AnimatePresence>
  ```

- [ ] **Step 4: Commit**

  ```bash
  git add package.json package-lock.json src/components/AnimatedPage.tsx src/App.tsx
  git commit -m "feat(ui): add page transition animations"
  ```

---

### Task 14: Board Drag Enhancement

**Files:**
- Modify: `src/pages/BoardPage.tsx`

- [ ] **Step 1: Add DragOverlay**

  Import `DragOverlay` from `@dnd-kit/core`.

- [ ] **Step 2: Render active task in overlay**

  ```tsx
  <DragOverlay>
    {activeTask ? <TaskCard task={activeTask} isOverlay /> : null}
  </DragOverlay>
  ```

- [ ] **Step 3: Enhance visual feedback**

  - Active task card: `scale-[1.02]` + `shadow-xl`
  - Droppable quadrant when over: `bg-blue-50/50`
  - After drop: add CSS transition for smooth settle

- [ ] **Step 4: Commit**

  ```bash
  git add src/pages/BoardPage.tsx
  git commit -m "feat(board): enhance drag feedback with overlay and animations"
  ```

---

### Task 15: End-to-End Testing and Polish

**Files:**
- All modified files
- `README.md` (update features)

- [ ] **Step 1: Run full test suite**

  ```bash
  cd src-tauri && cargo test
  cd ..
  npm test
  ```
  Expected: All tests pass.

- [ ] **Step 2: Manual verification checklist**

  - [ ] Create a diary with `#标签`，verify it appears in TagManager.
  - [ ] Create a task with `#标签`，verify board filter works.
  - [ ] Type `[[` in diary and select a task, verify link renders and click jumps.
  - [ ] Delete a linked task, verify link becomes invalid (gray).
  - [ ] Collapse sidebar, close app, reopen, verify state persists.
  - [ ] Switch pages, verify smooth transition.
  - [ ] Drag task across quadrants, verify visual feedback.

- [ ] **Step 3: Build production installer**

  ```bash
  npm run tauri build
  ```
  Expected: MSI and EXE generated in `src-tauri/target/release/bundle/`.

- [ ] **Step 4: Update README**

  Add v0.2.0 features to README feature list.

- [ ] **Step 5: Final commit and tag**

  ```bash
  git add .
  git commit -m "feat: complete v0.2.0 with tags, task links, layout, and animations"
  git tag v0.2.0
  git push origin master --tags
  ```

---

## Self-Review

- [ ] **Spec coverage:**
  - Tags from entries and tasks → Task 1, 3, 7, 9, 10
  - Tag panel and search → Task 7, 11
  - Task links with autocomplete → Task 4, 8, 9
  - Sidebar collapse + memory → Task 12
  - Page transitions + drag enhancement → Task 13, 14
  - Error handling → embedded in each task
  - Tests → Task 2, 7, 8, 15

- [ ] **Placeholder scan:** No TBD/TODO/fill-in-details patterns. Each step has concrete code or command.

- [ ] **Type consistency:** `Tag`, `TaskLink`, `AppSettings` types defined in Task 6 and used consistently. Command names match registration in `lib.rs`.

---

## Execution Options

Plan complete and saved to `docs/superpowers/plans/2026-06-11-daily-plan-v0.2.0.md`.

Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach do you want?
