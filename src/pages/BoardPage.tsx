import { useEffect, useMemo, useState } from 'react';
import {
  DndContext,
  DragEndEvent,
  useDraggable,
  useDroppable,
} from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import type { Task } from '@/types';

const quadrantLabels: Record<number, { label: string; color: string }> = {
  1: { label: '重要紧急', color: 'bg-red-50 border-red-200 dark:bg-red-950/30 dark:border-red-900' },
  2: { label: '重要不紧急', color: 'bg-blue-50 border-blue-200 dark:bg-blue-950/30 dark:border-blue-900' },
  3: { label: '紧急不重要', color: 'bg-yellow-50 border-yellow-200 dark:bg-yellow-950/30 dark:border-yellow-900' },
  4: { label: '不紧急不重要', color: 'bg-gray-50 border-gray-200 dark:bg-gray-900/30 dark:border-gray-800' },
};

function TaskCard({ task, onFinish, onDelete }: { task: Task; onFinish: (id: number) => void; onDelete: (id: number) => void }) {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({ id: `task-${task.id}`, data: task });
  const style = { transform: CSS.Translate.toString(transform), opacity: isDragging ? 0.5 : 1 };

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={style}
      className={`p-2 mb-2 border rounded bg-white dark:bg-card cursor-move ${task.status === 'completed' ? 'opacity-50 line-through' : ''}`}
    >
      <div className="font-medium text-sm">{task.title}</div>
      <div className="text-xs text-muted-foreground truncate">{task.description}</div>
      <div className="flex gap-2 mt-1">
        {task.status !== 'completed' && (
          <button className="text-xs text-green-600 hover:underline" onPointerDown={(e) => e.stopPropagation()} onClick={(e) => { e.stopPropagation(); onFinish(task.id); }}>完成</button>
        )}
        <button className="text-xs text-red-600 hover:underline" onPointerDown={(e) => e.stopPropagation()} onClick={(e) => { e.stopPropagation(); onDelete(task.id); }}>删除</button>
      </div>
    </div>
  );
}

function QuadrantColumn({ quadrant, tasks, onFinish, onDelete }: { quadrant: number; tasks: Task[]; onFinish: (id: number) => void; onDelete: (id: number) => void }) {
  const { setNodeRef, isOver } = useDroppable({ id: `quadrant-${quadrant}` });
  const info = quadrantLabels[quadrant];

  return (
    <div
      ref={setNodeRef}
      className={`flex-1 border rounded-lg p-3 flex flex-col ${info.color} ${isOver ? 'ring-2 ring-primary' : ''}`}
    >
      <div className="font-semibold text-sm mb-2 flex justify-between items-center">
        <span>{info.label}</span>
        <span className="text-xs text-muted-foreground">{tasks.length}</span>
      </div>
      <div className="flex-1 overflow-auto">
        {tasks.length === 0 && (
          <div className="text-xs text-muted-foreground text-center py-4">暂无任务</div>
        )}
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} onFinish={onFinish} onDelete={onDelete} />
        ))}
      </div>
    </div>
  );
}

export function BoardPage() {
  const [tasks, setTasks] = useState<Task[]>([]);

  const loadTasks = () => api.getTasks().then((data) => { if (data) setTasks(data); }).catch((err) => console.error(err));

  useEffect(() => {
    loadTasks();
  }, []);

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) return;
    const taskId = Number(String(active.id).replace('task-', ''));
    if (isNaN(taskId)) return;
    const quadrant = Number(String(over.id).replace('quadrant-', ''));
    if (quadrant >= 1 && quadrant <= 4) {
      api.moveTaskQuadrant(taskId, quadrant).then(loadTasks).catch((err) => console.error(err));
    }
  };

  const addTask = (quadrant: number) => {
    const title = prompt('任务标题');
    if (!title) return;
    const description = prompt('任务描述（可选）') || '';
    api.addTask(title, description, quadrant).then(loadTasks).catch((err) => console.error(err));
  };

  const finishTask = (id: number) => {
    const link = confirm('是否将任务引用到今天的记录？');
    api.finishTask(id).then(() => {
      loadTasks();
      if (link) {
        alert('任务已完成，请手动在记录页引用');
      }
    }).catch((err) => console.error(err));
  };

  const deleteTask = (id: number) => {
    if (confirm('确定删除此任务？')) {
      api.removeTask(id).then(loadTasks).catch((err) => console.error(err));
    }
  };

  const tasksByQuadrant = useMemo(() => {
    const map: Record<number, Task[]> = { 1: [], 2: [], 3: [], 4: [] };
    tasks.forEach((t) => { if (map[t.quadrant]) map[t.quadrant].push(t); });
    return map;
  }, [tasks]);

  return (
    <div className="h-full flex flex-col">
      <h2 className="text-xl font-bold mb-4">四象限任务看板</h2>
      <DndContext onDragEnd={handleDragEnd}>
        <div className="flex-1 grid grid-cols-2 grid-rows-2 gap-4 min-h-0">
          {[1, 2, 3, 4].map((q) => (
            <QuadrantColumn key={q} quadrant={q} tasks={tasksByQuadrant[q]} onFinish={finishTask} onDelete={deleteTask} />
          ))}
        </div>
      </DndContext>
      <div className="mt-4 flex gap-2">
        {[1, 2, 3, 4].map((q) => (
          <Button key={q} size="sm" variant="outline" onClick={() => addTask(q)}>
            添加至{quadrantLabels[q].label}
          </Button>
        ))}
      </div>
    </div>
  );
}
