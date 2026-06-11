export interface Entry {
  id: number;
  date: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: number;
  title: string;
  description: string;
  quadrant: 1 | 2 | 3 | 4;
  status: "active" | "completed" | "archived";
  created_at: string;
  completed_at: string | null;
  updated_at: string;
}

export interface SearchResult {
  result_type: "entry" | "task";
  id: number;
  date: string | null;
  content: string;
}
