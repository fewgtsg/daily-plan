import { useEffect, useState, useCallback, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import { format, addDays, subDays } from 'date-fns';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import { defaultTemplates } from '@/lib/templates';
import { TaskPanel } from '@/components/TaskPanel';
import type { Task } from '@/types';

export function RecordPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const dateParam = searchParams.get('date') || format(new Date(), 'yyyy-MM-dd');
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const requestedDateRef = useRef(dateParam);

  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({ placeholder: '开始记录今天的计划与复盘...' }),
    ],
    content: '',
    onUpdate: ({ editor }) => {
      debouncedSave(editor.getHTML());
    },
  });

  const debouncedSave = useCallback(
    (content: string) => {
      clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(() => {
        setSaving(true);
        api.saveEntry(dateParam, content)
          .then(() => setSaving(false))
          .catch(() => setSaving(false));
      }, 1000);
    },
    [dateParam]
  );

  useEffect(() => {
    return () => clearTimeout(saveTimerRef.current);
  }, []);

  useEffect(() => {
    setLoading(true);
    requestedDateRef.current = dateParam;
    api.getEntry(dateParam).then((e) => {
      if (requestedDateRef.current === dateParam && editor) {
        editor.commands.setContent(e?.content || '');
      }
    }).finally(() => setLoading(false));
  }, [dateParam, editor]);

  const insertTemplate = (content: string) => {
    editor?.chain().focus().insertContent(content).run();
  };

  const insertTask = (task: Task) => {
    const text = `> **任务**: ${task.title}\n> ${task.description}\n\n`;
    editor?.chain().focus().insertContent(text).run();
  };

  const goDay = (offset: number) => {
    const current = new Date(dateParam + 'T00:00:00');
    const next = offset > 0 ? addDays(current, offset) : subDays(current, Math.abs(offset));
    setSearchParams({ date: format(next, 'yyyy-MM-dd') });
  };

  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col">
        {loading && <div className="text-sm text-muted-foreground mb-2">加载中...</div>}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => goDay(-1)}>前一天</Button>
            <h2 className="text-xl font-bold">{dateParam}</h2>
            <Button variant="outline" size="sm" onClick={() => goDay(1)}>后一天</Button>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">{saving ? '保存中...' : '已自动保存'}</span>
            <select
              className="text-sm border rounded px-2 py-1 bg-background"
              onChange={(e) => {
                const tpl = defaultTemplates.find((t) => t.id === e.target.value);
                if (tpl) insertTemplate(tpl.content);
                e.target.value = '';
              }}
            >
              <option value="">插入模板</option>
              {defaultTemplates.map((t) => (
                <option key={t.id} value={t.id}>{t.label}</option>
              ))}
            </select>
          </div>
        </div>
        <div className="flex-1 border rounded-lg p-4 bg-card">
          <EditorContent editor={editor} className="prose dark:prose-invert max-w-none h-full outline-none" />
        </div>
      </div>
      <TaskPanel onInsertTask={insertTask} />
    </div>
  );
}
