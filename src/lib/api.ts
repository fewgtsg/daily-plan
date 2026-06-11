import { invoke } from "@tauri-apps/api/core";
import type { Entry, Task, SearchResult } from "../types";

let toastCallback: ((msg: string) => void) | null = null;
export function setToastCallback(cb: (msg: string) => void) {
  toastCallback = cb;
}

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    const msg = String(err);
    if (toastCallback) toastCallback(msg);
    else console.error(msg);
    return null;
  }
}

export const api = {
  getEntry: (date: string): Promise<Entry | null> => safeInvoke("get_entry", { date }),
  saveEntry: (date: string, content: string): Promise<Entry | null> =>
    safeInvoke("save_entry", { date, content }),
  getEntryDates: (): Promise<string[] | null> => safeInvoke("get_entry_dates"),

  getTasks: (status?: string): Promise<Task[] | null> => safeInvoke("get_tasks", { status }),
  addTask: (title: string, description: string, quadrant: number): Promise<Task | null> =>
    safeInvoke("add_task", { title, description, quadrant }),
  editTask: (id: number, title: string, description: string, quadrant: number): Promise<null> =>
    safeInvoke("edit_task", { id, title, description, quadrant }),
  finishTask: (id: number): Promise<null> => safeInvoke("finish_task", { id }),
  removeTask: (id: number): Promise<null> => safeInvoke("remove_task", { id }),
  moveTaskQuadrant: (id: number, quadrant: number): Promise<null> =>
    safeInvoke("move_task_quadrant", { id, quadrant }),

  search: (query: string): Promise<SearchResult[] | null> => safeInvoke("search", { query: `"${query}"` }),
};
