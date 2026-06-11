import { useEffect, useState } from 'react';
import { api } from '@/lib/api';
import type { Task } from '@/types';
import { Button } from '@/components/ui/button';

interface TaskPanelProps {
  onInsertTask: (task: Task) => void;
}

export function TaskPanel({ onInsertTask }: TaskPanelProps) {
  const [tasks, setTasks] = useState<Task[]>([]);

  useEffect(() => {
    api.getTasks('active').then((data) => { if (data) setTasks(data); }).catch((err) => console.error(err));
  }, []);

  const byQuadrant = (q: number) => tasks.filter((t) => t.quadrant === q);

  return (
    <div className="w-64 border-l p-4 bg-muted/30 overflow-auto">
      <h3 className="font-semibold mb-3">活跃任务</h3>
      {[1, 2, 3, 4].map((q) => (
        <div key={q} className="mb-4">
          <div className="text-xs text-muted-foreground mb-1">象限 {q}</div>
          {byQuadrant(q).map((task) => (
            <div key={task.id} className="flex items-center justify-between text-sm mb-1">
              <span className="truncate flex-1">{task.title}</span>
              <Button size="sm" variant="ghost" onClick={() => onInsertTask(task)}>
                引用
              </Button>
            </div>
          ))}
          {byQuadrant(q).length === 0 && <div className="text-xs text-muted-foreground">无任务</div>}
        </div>
      ))}
    </div>
  );
}
